use glam::Vec3A;
use std::iter::Peekable;
use crate::loader::file_format::{Background, Camera, Light, Material, Object, Scene};

pub fn parse_ray_file(contents: &str) -> Scene {
    let mut scene = Scene::default();
    let all_tokens: Vec<&str> = contents
        .lines()
        .flat_map(|line| {
            let line_without_comments = line.split('#').next().unwrap_or("").trim();
            line_without_comments.split_whitespace()
        })
        .collect();

    let mut tokens = all_tokens.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token {
            "Background" => scene.background = parse_background(&mut tokens),
            "Camera" => scene.camera = parse_camera(&mut tokens),
            "Lights" => scene.lights = parse_lights(&mut tokens),
            "Materials" => scene.materials = parse_materials(&mut tokens),
            "Group" => scene.objects = parse_group(&mut tokens),
            _ => {}
        }
    }

    scene
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
            _ => { tokens.next(); } // ignore unknown tokens
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
                camera.eye = parse_vec3a(tokens);
            }
            "lookAt" => {
                tokens.next();
                camera.look_at = parse_vec3a(tokens);
            }
            "up" => {
                tokens.next();
                camera.up = parse_vec3a(tokens);
            }
            "fovy" => {
                tokens.next();
                camera.fovy = parse_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => { tokens.next(); }
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
            _ => { tokens.next(); }
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
                light.position = parse_vec3a(tokens);
            }
            "color" => {
                tokens.next();
                light.color = parse_vec3a(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => { tokens.next(); }
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
            _ => { tokens.next(); }
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
            "diffuseColor" => {
                tokens.next();
                material.diffuse_color = parse_vec3a(tokens);
            }
            "specularColor" => {
                tokens.next();
                material.specular_color = parse_vec3a(tokens);
            }
            "reflectiveColor" => {
                tokens.next();
                material.reflective_color = parse_vec3a(tokens);
            }
            "shininess" => {
                tokens.next();
                material.shininess = parse_f32(tokens);
            }
            "transparentColor" => {
                tokens.next();
                material.transparent_color = parse_vec3a(tokens);
            }
            "indexOfRefraction" => {
                tokens.next();
                material.index_of_refraction = parse_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => { tokens.next(); }
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
            _ => { tokens.next(); }
        }
    }
    objects
}

fn parse_sphere<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Object {
    let mut material_index = 0;
    let mut center = Vec3A::ZERO;
    let mut radius = 0.0;
    expect_token(tokens, "{");
    while let Some(token) = tokens.peek() {
        match *token {
            "materialIndex" => {
                tokens.next();
                material_index = parse_usize(tokens);
            }
            "center" => {
                tokens.next();
                center = parse_vec3a(tokens);
            }
            "radius" => {
                tokens.next();
                radius = parse_f32(tokens);
            }
            "}" => {
                tokens.next();
                break;
            }
            _ => { tokens.next(); }
        }
    }
    Object::Sphere { material_index, center, radius }
}


fn parse_vec3a<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Vec3A {
    let x = parse_f32(tokens);
    let y = parse_f32(tokens);
    let z = parse_f32(tokens);
    Vec3A::new(x, y, z)
}

fn parse_f32<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> f32 {
    tokens.next().unwrap().parse().unwrap()
}

fn parse_usize<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> usize {
    tokens.next().unwrap().parse().unwrap()
}

fn expect_token<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>, expected: &str) {
    let next_token = tokens.next();
    if next_token != Some(expected) {
        // A simple way to handle errors, you might want to implement more robust error handling
        panic!("Expected token '{}', found {:?}", expected, next_token);
    }
}
