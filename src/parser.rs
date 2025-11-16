use glam::Vec3A;
use crate::file_format::*;
use std::iter::Peekable;
use std::str::SplitWhitespace;

pub fn parse_ray_file(contents: &str) -> Scene {
    let mut scene = Scene::default();
    let mut tokens = contents.split_whitespace().peekable();

    while let Some(token) = tokens.next() {
        match token {
            "#" => {
                // Comment, skip until newline
                let remaining = tokens.next().unwrap_or("").to_string();
                let mut line = remaining;
                while !line.contains('\n') {
                    line = tokens.next().unwrap_or("").to_string();
                }
            }
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

fn parse_background(tokens: &mut Peekable<SplitWhitespace>) -> Background {
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

fn parse_camera(tokens: &mut Peekable<SplitWhitespace>) -> Camera {
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

fn parse_lights(tokens: &mut Peekable<SplitWhitespace>) -> Vec<Light> {
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

fn parse_light(tokens: &mut Peekable<SplitWhitespace>) -> Light {
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

fn parse_materials(tokens: &mut Peekable<SplitWhitespace>) -> Vec<Material> {
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

fn parse_material(tokens: &mut Peekable<SplitWhitespace>) -> Material {
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

fn parse_group(tokens: &mut Peekable<SplitWhitespace>) -> Vec<Object> {
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

fn parse_sphere(tokens: &mut Peekable<SplitWhitespace>) -> Object {
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


fn parse_vec3a(tokens: &mut Peekable<SplitWhitespace>) -> Vec3A {
    let x = parse_f32(tokens);
    let y = parse_f32(tokens);
    let z = parse_f32(tokens);
    Vec3A::new(x, y, z)
}

fn parse_f32(tokens: &mut Peekable<SplitWhitespace>) -> f32 {
    tokens.next().unwrap().parse().unwrap()
}

fn parse_usize(tokens: &mut Peekable<SplitWhitespace>) -> usize {
    tokens.next().unwrap().parse().unwrap()
}

fn expect_token(tokens: &mut Peekable<SplitWhitespace>, expected: &str) {
    let next_token = tokens.next();
    if next_token != Some(expected) {
        // A simple way to handle errors, you might want to implement more robust error handling
        panic!("Expected token '{}', found {:?}", expected, next_token);
    }
}
