extern crate ascia;

use std::cell::{RefCell, RefMut};
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
use ascia::ascia::core::{CParticle, LambertMaterial, LambertWithShadowMaterial, Local, Material, ObjectNode, ObjectNodeAttribute, ObjectNodeAttributeDispatcher, Polygon, PresetAsciaEnvironment, PresetCamera, PresetLight, PresetPolygonMaterial, PresetObjectNodeAttributeDispatcher, AsciaEngine};
use ascia::ascia::core::CParticleMode::SPHERE;
use ascia::ascia::lights::PointLight;
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;
use ascia::ascia::util::{available_preset_cameras, move_camera, preset_camera_info, rotate_camera, TermiosController};

fn main() {
    let args: Vec<String> = env::args().collect();

    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };

    let fps_upper_limit :u64 = 30;

    let mut engine = AsciaEngine::<PresetAsciaEnvironment>::new(width, height);

    let cameras = available_preset_cameras();
    let mut now_camera_index = 1usize;

    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5, 1.0);

    cam_objn.attribute = cameras[now_camera_index].clone();

    engine.genesis_local.add_child(cam_objn);

    let mut cube_objn = ObjectNode::from("cube",PrimitiveGenerator::cube(200.0, PresetPolygonMaterial::LambertMaterial(Default::default())));
    cube_objn.position.z = 400.0;
    engine.genesis_local.add_child(cube_objn);

    let mut null_container = ObjectNode::new("null container");
    null_container.position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };

    for i in 0..1{
        let mut pot = ObjectNode::from(&format!("teapot {}", i), load_teapot("./examples/teapot_bezier1.tris.txt",&PresetPolygonMaterial::LambertWithShadowMaterial(
            LambertWithShadowMaterial{
                color: ColorRGBf32{
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                priority: 10,
            }
        )));

        pot.position = Vec3{
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }.rotate_by(&Quaternion::new(&Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },i as f32 * 0.5, 1.0));

        pot.direction = Quaternion::new(&Vec3{
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }, 0.0, 10.0);
        null_container.add_child(pot);
    }

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
    light_objn.position = Vec3{
        x: 0.0,
        y: 50.0,
        z: -100.0,
    };
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
                    now_camera_index = (now_camera_index + 2) % cameras.len();
                    camera.attribute = cameras[now_camera_index].clone()
                },
                _ => {}
            }
        }
    }).unwrap();

    let mut last_time = Instant::now();

    for _i in 0..65536 {
        engine.sync_engine_time();
        engine.genesis_local.child_mut("null container").unwrap().direction = Quaternion::new(&Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }, engine.engine_time().as_secs_f32(), 1.0);


        termios_controller.input(&mut engine);
        engine.genesis_global = engine.genesis_local.generate_global_nodes();
        engine.render(engine.genesis_global.child("camera").unwrap());

        if (last_time.elapsed().as_millis() as u64) < (1000 / fps_upper_limit){
            thread::sleep(Duration::from_millis(1000 / fps_upper_limit - last_time.elapsed().as_millis() as u64));
        }

        let mut dur = last_time.elapsed();

        println!("dur:{}      ",dur.as_millis());
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", preset_camera_info(&engine.genesis_global.child("camera").unwrap().attribute));
        last_time = Instant::now();
    }
}



// https://users.cs.utah.edu/~dejohnso/models/teapot.html
fn load_teapot(path:&str, material:&PresetPolygonMaterial) -> Vec<Polygon<PresetAsciaEnvironment, Local>>{
    let mut polygons:Vec<Polygon<PresetAsciaEnvironment, Local>> = vec![];

    if let Ok(f) = File::open(path){
        let mut reader = BufReader::new(f);
        let mut s = String::new();
        if let Ok(len) = reader.read_line(&mut s){
            if let Ok(num_polygon) = usize::from_str(&(s.trim())){
                let mut m = Matrix33{
                    v1: Default::default(),
                    v2: Default::default(),
                    v3: Default::default(),
                };
                for i in 0..num_polygon{
                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");

                    m.v1.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v1.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v1.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");
                    m.v2.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v2.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v2.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    s.clear();
                    reader.read_line(&mut s);
                    let mut vs = s.split(" ");
                    m.v3.x = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v3.y = f32::from_str(vs.next().unwrap().trim()).unwrap();
                    m.v3.z = f32::from_str(vs.next().unwrap().trim()).unwrap();

                    reader.read_line(&mut s);

                    polygons.push(Polygon{
                        poses: m,
                        material: material.clone(),
                        _ph: Default::default(),
                    });
                }
            }
            else{
                return vec![];
            }
        }
        else{
            return vec![];
        }
    }
    return polygons;
}