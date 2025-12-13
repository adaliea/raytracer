use crate::loader::file_format::{
    Animatable, Animation, AnimationSettings, Background, Camera, Keyframe, Light, Material,
    Object, Scene,
};
use glam::{Vec2, Vec3A};
use splines::Interpolation;
use std::iter::Peekable;

pub fn parse_ray_file(contents: &str) -> Scene {
    let mut scene = Scene::default();
    let all_tokens: Vec<String> = contents
        .lines()
        .flat_map(|line| {
            let line_without_comments = line.split('#').next().unwrap_or("").trim();
            line_without_comments
                .replace('{', " { ")
                .replace('}', " } ")
                .replace('(', " ( ")
                .replace(')', " ) ")
                .replace(',', " , ")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();

    let mut tokens = all_tokens.iter().map(|s| s.as_str()).peekable();
    while let Some(token) = tokens.next() {
        match token {
            "Background" => scene.background = parse_background(&mut tokens),
            "Camera" => scene.camera = parse_camera(&mut tokens),
            "Lights" => scene.lights = parse_lights(&mut tokens),
            "Materials" => scene.materials = parse_materials(&mut tokens),
            "Group" => scene.objects = parse_group(&mut tokens),
            "AnimationSettings" => scene.animation_settings = parse_animation_settings(&mut tokens),
            _ => {}
        }
    }

    scene
}

fn parse_animation_settings<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> AnimationSettings {
    let mut settings = AnimationSettings::default();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "duration" => {
                tokens.next();
                settings.duration = parse_f32(tokens);
            }
            "frameRate" => {
                tokens.next();
                settings.frame_rate = parse_u32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    settings
}

fn parse_background<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Background {
    let mut background = Background::default();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "color" => {
                tokens.next(); // consume 'color'
                background.color = parse_vec3a(tokens);
            }
            "ambientLight" => {
                tokens.next(); // consume 'ambientLight'
                background.ambient_light = parse_vec3a(tokens);
            }
            "}" => {
                tokens.next(); // consume '}'
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    background
}

fn parse_camera<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Camera {
    let mut camera = Camera::default();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "eye" => {
                tokens.next();
                camera.eye = parse_animatable_vec3a(tokens);
            }
            "lookAt" => {
                tokens.next();
                camera.look_at = parse_animatable_vec3a(tokens);
            }
            "up" => {
                tokens.next();
                camera.up = parse_animatable_vec3a(tokens);
            }
            "fovy" => {
                tokens.next();
                camera.fovy = parse_animatable_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    camera
}

fn parse_lights<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec<Light> {
    let mut lights = Vec::new();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "Light" => {
                tokens.next();
                lights.push(parse_light(tokens));
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    lights
}

fn parse_light<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Light {
    let mut light = Light::default();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "position" => {
                tokens.next();
                light.position = parse_animatable_vec3a(tokens);
            }
            "color" => {
                tokens.next();
                light.color = parse_animatable_vec3a(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    light
}

fn parse_materials<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec<Material> {
    let mut materials = Vec::new();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "Material" => {
                tokens.next();
                materials.push(parse_material(tokens));
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    materials
}

fn parse_material<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Material {
    let mut material = Material::default();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "textureFilename" => {
                tokens.next();
                let filename = tokens.next().unwrap();
                if filename != "NULL" {
                    material.texture_filename = Some(filename.to_string());
                }
            }
            "normalMapFilename" => {
                tokens.next();
                let filename = tokens.next().unwrap();
                if filename != "NULL" {
                    material.normal_map_filename = Some(filename.to_string());
                }
            }
            "displacementMapFilename" => {
                tokens.next();
                let filename = tokens.next().unwrap();
                if filename != "NULL" {
                    material.displacement_map_filename = Some(filename.to_string());
                }
            }
            "diffuseColor" => {
                tokens.next();
                material.diffuse_color = parse_animatable_vec3a(tokens);
            }
            "specularColor" => {
                tokens.next();
                material.specular_color = parse_animatable_vec3a(tokens);
            }
            "reflectiveColor" => {
                tokens.next();
                material.reflective_color = parse_animatable_vec3a(tokens);
            }
            "shininess" => {
                tokens.next();
                material.shininess = parse_animatable_f32(tokens);
            }
            "transparentColor" => {
                tokens.next();
                material.transparent_color = parse_animatable_vec3a(tokens);
            }
            "indexOfRefraction" => {
                tokens.next();
                material.index_of_refraction = parse_animatable_f32(tokens);
            }
            "displacementStrength" => {
                tokens.next();
                material.displacement_strength = parse_animatable_f32(tokens);
            }
            "subdivisionLevel" => {
                tokens.next();
                material.subdivision_level = Some(parse_u32(tokens));
            }

            "maxEdgeLength" => {
                tokens.next();
                material.max_edge_length = Some(parse_animatable_f32(tokens));
            }
            "emissiveColor" => {
                tokens.next();
                material.emissive_color = Some(parse_animatable_vec3a(tokens));
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    material
}

fn parse_group<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec<Object> {
    let mut objects = Vec::new();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "Sphere" => {
                tokens.next();
                objects.push(parse_sphere(tokens));
            }
            "}" => {
                tokens.next();
                break;
            }
            "Triangle" => {
                tokens.next();
                objects.push(parse_triangle(tokens));
            }
            "Mesh" => {
                tokens.next();
                objects.push(parse_mesh(tokens));
            }
            "Cylinder" => {
                tokens.next();
                objects.push(parse_cylinder(tokens));
            }
            _ => {
                tokens.next();
            }
        }
    }
    objects
}

fn parse_triangle<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Object {
    let mut material_index = 0;
    let mut vertex0 = Animatable::Static(Vec3A::ZERO);
    let mut vertex1 = Animatable::Static(Vec3A::ZERO);
    let mut vertex2 = Animatable::Static(Vec3A::ZERO);
    let mut tex_xy_0 = Animatable::Static(Vec2::ZERO);
    let mut tex_xy_1 = Animatable::Static(Vec2::ZERO);
    let mut tex_xy_2 = Animatable::Static(Vec2::ZERO);
    let mut normal0 = Animatable::Static(Vec3A::ZERO);
    let mut normal1 = Animatable::Static(Vec3A::ZERO);
    let mut normal2 = Animatable::Static(Vec3A::ZERO);
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "materialIndex" => {
                tokens.next();
                material_index = parse_usize(tokens);
            }
            "vertex0" => {
                tokens.next();
                vertex0 = parse_animatable_vec3a(tokens);
            }
            "vertex1" => {
                tokens.next();
                vertex1 = parse_animatable_vec3a(tokens);
            }
            "vertex2" => {
                tokens.next();
                vertex2 = parse_animatable_vec3a(tokens);
            }
            "tex_xy_0" => {
                tokens.next();
                tex_xy_0 = parse_animatable_vec2(tokens);
            }
            "tex_xy_1" => {
                tokens.next();
                tex_xy_1 = parse_animatable_vec2(tokens);
            }
            "tex_xy_2" => {
                tokens.next();
                tex_xy_2 = parse_animatable_vec2(tokens);
            }
            "normal0" => {
                tokens.next();
                normal0 = parse_animatable_vec3a(tokens);
            }
            "normal1" => {
                tokens.next();
                normal1 = parse_animatable_vec3a(tokens);
            }
            "normal2" => {
                tokens.next();
                normal2 = parse_animatable_vec3a(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }

    if normal0 == Animatable::Static(Vec3A::ZERO)
        && normal1 == Animatable::Static(Vec3A::ZERO)
        && normal2 == Animatable::Static(Vec3A::ZERO)
    {
        if let (
            Animatable::Static(vertex0),
            Animatable::Static(vertex1),
            Animatable::Static(vertex2),
        ) = (vertex0.clone(), vertex1.clone(), vertex2.clone())
        {
            let v0v1 = vertex1 - vertex0;
            let v0v2 = vertex2 - vertex0;
            let face_normal = v0v1.cross(v0v2).normalize();
            normal0 = Animatable::Static(face_normal);
            normal1 = Animatable::Static(face_normal);
            normal2 = Animatable::Static(face_normal);
        }
    }

    Object::Triangle {
        material_index,
        vertex0,
        vertex1,
        vertex2,
        tex_xy_0,
        tex_xy_1,
        tex_xy_2,
        normal0,
        normal1,
        normal2,
    }
}

fn parse_sphere<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Object {
    let mut material_index = 0;
    let mut center = Animatable::Static(Vec3A::ZERO);
    let mut radius = Animatable::Static(0.0);
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "materialIndex" => {
                tokens.next();
                material_index = parse_usize(tokens);
            }
            "center" => {
                tokens.next();
                center = parse_animatable_vec3a(tokens);
            }
            "radius" => {
                tokens.next();
                radius = parse_animatable_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    Object::Sphere {
        material_index,
        center,
        radius,
    }
}

fn parse_mesh<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Object {
    let mut material_index = 0;
    let mut filename = String::new();
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "materialIndex" => {
                tokens.next();
                material_index = parse_usize(tokens);
            }
            "filename" => {
                tokens.next();
                filename = tokens.next().unwrap().to_string();
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    Object::Mesh {
        filename,
        material_index,
    }
}

fn parse_cylinder<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Object {
    let mut material_index = 0;
    let mut center = Animatable::Static(Vec3A::ZERO);
    let mut radius = Animatable::Static(0.0);
    let mut height = Animatable::Static(0.0);
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "materialIndex" => {
                tokens.next();
                material_index = parse_usize(tokens);
            }
            "center" => {
                tokens.next();
                center = parse_animatable_vec3a(tokens);
            }
            "radius" => {
                tokens.next();
                radius = parse_animatable_f32(tokens);
            }
            "height" => {
                tokens.next();
                height = parse_animatable_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => {
                tokens.next();
            }
        }
    }
    Object::Cylinder {
        material_index,
        center,
        radius,
        height,
    }
}

fn parse_animatable_vec3a<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Animatable<Vec3A> {
    if let Some(token) = tokens.peek() {
        match *token {
            "Bezier" | "CatmullRom" | "Linear" => {
                let interpolation_str = tokens.next().unwrap();
                let interpolation = match interpolation_str {
                    "Bezier" => Interpolation::Bezier(0.5),
                    "CatmullRom" => Interpolation::CatmullRom,
                    "Linear" => Interpolation::Linear,
                    _ => panic!("Unknown interpolation type"),
                };

                expect_token(tokens, "{");

                let mut keyframes = Vec::new();

                while tokens.peek() != Some(&"}") {
                    expect_token(tokens, "(");
                    let value = parse_vec3a(tokens);
                    expect_token(tokens, ")");
                    let time = parse_f32(tokens);
                    keyframes.push(Keyframe { value, time });

                    if tokens.peek() == Some(&",") {
                        tokens.next(); // consume ","
                    }
                }

                expect_token(tokens, "}");

                return Animatable::Animated(Animation {
                    interpolation,
                    keyframes,
                });
            }
            _ => {
                return Animatable::Static(parse_vec3a(tokens));
            }
        }
    }
    panic!("Unexpected end of tokens");
}

fn parse_animatable_vec2<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Animatable<Vec2> {
    if let Some(token) = tokens.peek() {
        match *token {
            "Bezier" | "CatmullRom" | "Linear" => {
                let interpolation_str = tokens.next().unwrap();
                let interpolation = match interpolation_str {
                    "Bezier" => Interpolation::Bezier(0.5),
                    "CatmullRom" => Interpolation::CatmullRom,
                    "Linear" => Interpolation::Linear,
                    _ => panic!("Unknown interpolation type"),
                };

                expect_token(tokens, "{");

                let mut keyframes = Vec::new();

                while tokens.peek() != Some(&"}") {
                    expect_token(tokens, "(");
                    let value = parse_vec2(tokens);
                    expect_token(tokens, ")");
                    let time = parse_f32(tokens);
                    keyframes.push(Keyframe { value, time });

                    if tokens.peek() == Some(&",") {
                        tokens.next(); // consume ","
                    }
                }

                expect_token(tokens, "}");

                return Animatable::Animated(Animation {
                    interpolation,
                    keyframes,
                });
            }
            _ => {
                return Animatable::Static(parse_vec2(tokens));
            }
        }
    }
    panic!("Unexpected end of tokens");
}

fn parse_animatable_f32<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Animatable<f32> {
    if let Some(token) = tokens.peek() {
        match *token {
            "Bezier" | "CatmullRom" | "Linear" => {
                let interpolation_str = tokens.next().unwrap();
                let interpolation = match interpolation_str {
                    "Bezier" => Interpolation::Bezier(0.5),
                    "CatmullRom" => Interpolation::CatmullRom,
                    "Linear" => Interpolation::Linear,
                    _ => panic!("Unknown interpolation type"),
                };

                expect_token(tokens, "{");

                let mut keyframes = Vec::new();

                while tokens.peek() != Some(&"}") {
                    let value = parse_f32(tokens);
                    let time = parse_f32(tokens);
                    keyframes.push(Keyframe { value, time });
                    if tokens.peek() == Some(&",") {
                        tokens.next(); // consume ","
                    }
                }

                expect_token(tokens, "}");

                return Animatable::Animated(Animation {
                    interpolation,
                    keyframes,
                });
            }
            _ => {
                return Animatable::Static(parse_f32(tokens));
            }
        }
    }
    panic!("Unexpected end of tokens");
}

fn parse_vec3a<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec3A {
    let x = parse_f32(tokens);
    let y = parse_f32(tokens);
    let z = parse_f32(tokens);
    Vec3A::new(x, y, z)
}

fn parse_vec2<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec2 {
    let x = parse_f32(tokens);
    let y = parse_f32(tokens);
    Vec2::new(x, y)
}

fn parse_f32<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> f32 {
    tokens.next().unwrap().parse().unwrap()
}

fn parse_usize<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> usize {
    tokens.next().unwrap().parse().unwrap()
}

fn parse_u32<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> u32 {
    tokens.next().unwrap().parse().unwrap()
}

fn expect_token<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>, expected: &str) {
    let next_token = tokens.next();
    if next_token != Some(expected) {
        // A simple way to handle errors, you might want to implement more robust error handling
        panic!("Expected token '{}', found {:?}", expected, next_token);
    }
}
