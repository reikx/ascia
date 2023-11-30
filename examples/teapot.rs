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
use ascia::ascia::camera::{BVHGPUCamera, SimpleBVHCamera, SimpleCamera};
use ascia::ascia::core::{AsciaEngine, CameraEntity, ColorRGBf32, ColorRGBu8, CParticle, Entity, Material, ObjectNode, Polygon, SimpleEntity};
use ascia::ascia::core::CParticleMode::SPHERE;
use ascia::ascia::core::MaterialMode::{LAMBERT, LAMBERT_WITH_SHADOW};
use ascia::ascia::lights::Light;
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let mut camera = create_camera_from_args(&engine);
    camera.borrow_mut().repr_refmut().direction = Quaternion::new(Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5);

    ObjectNode::connect(&engine.genesis,camera.borrow().repr());
    engine.event_handler.register_update_event(camera.clone());
    engine.event_handler.set_render_event(camera.clone());

    let mut plane = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(200.0, Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT_WITH_SHADOW,
        priority: 0,
    })));
    plane.repr_refmut().position.z = 400.0;
    plane.register_update_fn( |mut objn, d|{
        objn.direction = Quaternion::new(Vec3{
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }.normalize(),PI * d.as_secs_f32());
    });
    ObjectNode::connect(&engine.genesis,&plane.repr());
    engine.event_handler.register_update_event(Rc::new(RefCell::new(plane)));

    let mut null_container = SimpleEntity::new(ObjectNode::new());
    null_container.repr_refmut().position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };
    null_container.register_update_fn(|mut objn, d|{
        objn.direction = Quaternion::new(Vec3{
            x:1.0,
            y:0.0,
            z:0.0
        },1.00) * Quaternion::new(Vec3{
            x:0.0,
            y:1.0,
            z:0.0
        },(d.as_millis() as f32) / 5000.0 * 2.0 * PI);
    });
    ObjectNode::connect(&engine.genesis,&null_container.repr());

    for i in 0..2{
        let mut pot = SimpleEntity::new(ObjectNode::from(load_teapot("./examples/teapot_bezier2.tris.txt",&Material{
            color: ColorRGBf32{
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            mode: LAMBERT,
            priority: 1,
        })));

        pot.repr_refmut().position = Vec3{
            x: 40.0 * i as f32 - 20.0,
            y: 0.0,
            z: 0.0,
        }.rotate_by(&Quaternion::new(Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },i as f32 * 0.5));

        pot.repr_refmut().direction = Quaternion::new(Vec3{
            x: 0.0,
            y: 0.0,
            z: 3.0,
        },PI);

        pot.connect(&null_container);
        engine.event_handler.register_update_event(Rc::new(RefCell::new(pot)));
    }

    let mut light = Light::new(ColorRGBu8{
        r:255,
        g:255,
        b:255
    }.into(), 1.0);
    light.repr_refmut().position = Vec3{
        x: 0.0,
        y: 100.0,
        z: -100.0,
    };
    engine.lights.push(Rc::new(RefCell::new(light)));

    engine.event_handler.register_update_event(Rc::new(RefCell::new(null_container)));

    let mut input = create_stdin_controller().unwrap();

    let mut last_time = Instant::now();

    engine.viewport.clean();
    for _i in 0..65536 {
        move_camera(camera.borrow_mut().repr_refmut(),&mut input,2.0,1.0);

        engine.update();
        engine.render();

        while last_time.elapsed().as_millis() < 10{
            thread::sleep(Duration::from_millis(2));
        }
        let mut dur = last_time.elapsed();

        println!("dur:{}      ",dur.as_millis());
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll");
        last_time = Instant::now();
    }
}

fn create_camera_from_args(engine:&AsciaEngine) -> Rc<RefCell<dyn CameraEntity>> {
    let args: Vec<String> = env::args().collect();
    //let mut camera = SimpleCamera::new(3,3,&engine.wgpu_daq.as_ref().unwrap().0);
    let mut camera = SimpleBVHCamera::new(3,3,&engine.wgpu_daq.as_ref().unwrap().0);
    if args.contains(&String::from("--usegpu")) || cfg!(debug_assertions){
        camera.use_gpu = true;
    }
    else{
        camera.use_gpu = false;
    }
    return Rc::new(RefCell::new(camera));
}
fn create_stdin_controller() -> Option<File>{
    let rawfdstdin = stdin().as_raw_fd();
    if let Ok(mut termi) = termios::Termios::from_fd(rawfdstdin){
        termi.c_lflag &= !termios::os::target::ICANON;
        termi.c_lflag &= !termios::os::target::ECHO;
        termi.c_cc[termios::os::target::VMIN] = 0;
        termi.c_cc[termios::os::target::VTIME] = 0;

        if let Ok(result) = termios::tcsetattr(rawfdstdin, termios::os::target::TCSANOW, &mut termi){
            return Some(unsafe { File::from_raw_fd(rawfdstdin) });
        }
        else{
            return None;
        }
    }
    else{
        return None;
    }
}

fn move_camera(mut cam_objn:RefMut<ObjectNode>, input: &mut File, velocity:f32, rotation_speed:f32){
    let mut v = vec![0;1];
    if let Err(e) = input.read_to_end(&mut v){
        println!("{}",e);
        panic!();
    }
    for c in v.iter(){
        let p = cam_objn.position.clone();
        let d = cam_objn.direction.clone();
        if *c == b'w'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: velocity,
                y: 0.0,
                z: 0.0,
            });
        }
        else if *c == b's'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: -velocity,
                y: 0.0,
                z: 0.0,
            });
        }
        else if *c == b'a'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: 0.0,
                z: velocity,
            });
        }
        else if *c == b'd'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: 0.0,
                z: -velocity,
            });
        }
        else if *c == b'g'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: -velocity,
                z: 0.0,
            });
        }
        else if *c == b'h'{
            cam_objn.position = p + d.rotate(&Vec3{
                x: 0.0,
                y: velocity,
                z: 0.0,
            });
        }
        else if *c == b'i'{
            cam_objn.direction = d * Quaternion::new(Vec3{
                x: 0.0,
                y: 0.0,
                z: rotation_speed,
            },PI * 0.02);
        }
        else if *c == b'k'{
            cam_objn.direction = d * Quaternion::new(Vec3{
                x: 0.0,
                y: 0.0,
                z: rotation_speed,
            },PI * -0.02);
        }
        else if *c == b'j'{
            cam_objn.direction = Quaternion::new(Vec3{
                x: 0.0,
                y: rotation_speed,
                z: 0.0,
            },PI * -0.02) * d;
        }
        else if *c == b'l'{
            cam_objn.direction = Quaternion::new(Vec3{
                x: 0.0,
                y: rotation_speed,
                z: 0.0,
            },PI * 0.02) * d;
        }
    }
}


// https://users.cs.utah.edu/~dejohnso/models/teapot.html
fn load_teapot(path:&str, material:&Material) -> Vec<Polygon>{
    let mut polygons:Vec<Polygon> = vec![];

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