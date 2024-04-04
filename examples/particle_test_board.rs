extern crate ascia;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::fs::{File, read};
use std::{env, io, thread};
use std::io::{BufRead, BufReader, Read, stdin};
use std::ops::Add;
use std::os::fd::{AsRawFd, FromRawFd};
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use ascia::ascia::camera::{NaiveBVH, SimpleBVHCamera, SimpleCamera};
use ascia::ascia::color::{ColorRGBf32, ColorRGBu8};
use ascia::ascia::core::{CParticle, LambertMaterial, Local, Material, ObjectNode, ObjectNodeAttribute, ObjectNodeAttributeDispatcher, Polygon, PresetAsciaEnvironment, PresetCamera, PresetLight, PresetPolygonMaterial, PresetObjectNodeAttributeDispatcher, AsciaEngine, PresetCParticleMaterial, FlatMaterial};
use ascia::ascia::core::CParticleMode::ARG;
use ascia::ascia::lights::{PointLight};
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;
use ascia::ascia::util::{available_preset_cameras, move_camera, preset_camera_info, rotate_camera, TermiosController};

fn main() {
    let args: Vec<String> = env::args().collect();

    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };

    let fps_upper_limit :u64 = 30;

    let mut engine = AsciaEngine::<PresetAsciaEnvironment>::new(width, height);

    let mut cameras = available_preset_cameras();
    let mut now_camera_index = 0usize;

    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5, 1.0);
    cam_objn.attribute = cameras[now_camera_index].clone();

    engine.genesis_local.add_child(cam_objn);

    let mut null_container = ObjectNode::new("null container");
    null_container.position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };

    let mut en1 = ObjectNode::new("en1");

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    for i in 0..2048{
        en1.c_particles.push(CParticle{
            position: Vec3{
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            },
            velocity: Default::default(),
            c: lorem.chars().nth(i % lorem.len()).unwrap(),
            threshold: PI / 400.0,
            mode: ARG,
            material: PresetCParticleMaterial::FlatMaterial(FlatMaterial{
                color: ColorRGBf32{
                    r: 1.0,
                    g: if i % 2 == 0 { 0.0 } else { 1.0 },
                    b: 1.0,
                },
                priority: 0,
            }),
            _ph: Default::default(),
        });
    }

    null_container.add_child(en1);
    engine.genesis_local.add_child(null_container);

    let light = PointLight{
        color: ColorRGBu8 {
            r:255,
            g:255,
            b:255
        }.into(),
        power: 1.0,
    };

    let mut light_objn = ObjectNode::new("light");
    light_objn.position.y = 100.0;
    light_objn.position.z = -100.0;
    light_objn.attribute = PresetObjectNodeAttributeDispatcher::from(light).make_shared();
    engine.genesis_local.add_child(light_objn);

    let mut termios_controller = TermiosController::generate(|c, e|{
        if let Some(camera) = e.genesis_local.child_mut("camera"){
            match c {
                b'w' => move_camera(3.0, camera, &Vec3{ x: 1.0, y: 0.0, z:0.0 }),
                b's' => move_camera(3.0, camera, &Vec3{ x: -1.0, y: 0.0, z:0.0 }),
                b'a' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: 0.0, z:1.0 }),
                b'd' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: 0.0, z:-1.0 }),
                b'h' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: 1.0, z:0.0 }),
                b'g' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: -1.0, z:0.0 }),
                b'i' => rotate_camera(0.1, camera, &Vec3{ x: 0.0, y: 0.0, z:1.0 }),
                b'k' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: 0.0, z:-1.0 }),
                b'j' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: -1.0, z: 0.0 }),
                b'l' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: 1.0, z: 0.0 }),
                b'v' => {
                    now_camera_index = (now_camera_index + 1) % cameras.len();
                    camera.attribute = cameras[now_camera_index].clone()
                },
                _ => {}
            }
        }
    }).unwrap();

    let mut last_time = Instant::now();

    loop {
        engine.sync_engine_time();
        let engine_time = engine.engine_time();

        let d = 0.5 + f32::sin(engine_time.as_secs_f32() * 0.3 * PI) * 0.5;
        for i in 0..2048 {
            engine.genesis_local.child_mut("null container").unwrap().child_mut("en1").unwrap().c_particles[i].position = d * Vec3 {
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            } + (1.0 - d) * Vec3 {
                x: 80.0 * f32::cos(i as f32 * 0.1) - 40.0,
                y: 1.0 * i as f32 - 10.0,
                z: 100.0 + 80.0 * f32::sin(i as f32 * 0.1)
            };
        }

        termios_controller.input(&mut engine);
        engine.update_global_nodes();
        engine.render(engine.genesis_global.child("camera").unwrap());

        if (last_time.elapsed().as_millis() as u64) < (1000 / fps_upper_limit){
            thread::sleep(Duration::from_millis(1000 / fps_upper_limit - last_time.elapsed().as_millis() as u64));
        }

        let mut dur = last_time.elapsed();

        println!("dur:{}      ", dur.as_millis());
        println!("fps:{}      ", 1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", preset_camera_info(&engine.genesis_global.child("camera").unwrap().attribute));

        last_time = Instant::now();
    }
}
