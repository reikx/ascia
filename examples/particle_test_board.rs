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
use ascia::ascia::camera::{BVHGPUCamera, SimpleCamera};
use ascia::ascia::core::{AsciaEngine, ColorRGBf32, ColorRGBu8, CParticle, Entity, Material, ObjectNode, Polygon, SimpleEntity};
use ascia::ascia::core::CParticleMode::ARG;
use ascia::ascia::core::MaterialMode::LAMBERT;
use ascia::ascia::lights::Light;
use ascia::ascia::math::{Matrix33, Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

// https://users.cs.utah.edu/~dejohnso/models/teapot.html

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    /*
    let mut camera = SimpleCamera::new(1,&engine.wgpu_daq.as_ref().unwrap().0);
    if args.contains(&String::from("--usegpu")) || cfg!(debug_assertions){
        camera.use_gpu = true;
    }
    camera.use_gpu = false;
*/
    let mut camera = BVHGPUCamera::new(3,3,&engine.wgpu_daq.as_ref().unwrap().0);

    let cam_objn = camera.repr().clone();
    RefCell::borrow_mut(&cam_objn).direction = Quaternion::new(Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5);

    ObjectNode::connect(&engine.genesis,&cam_objn);
    {
        let rr = Rc::new(RefCell::new(camera));
        engine.event_handler.register_update_event(rr.clone());
        engine.event_handler.set_render_event(rr);
    }

    let mut null_container = SimpleEntity::new(ObjectNode::new());
    null_container.objn_refmut().position = Vec3{
        x:0.0,
        y:0.0,
        z:100.0
    };
    null_container.register_update_fn(|mut objn, d|{
        objn.direction = Quaternion::new(Vec3{
            x:0.0,
            y:1.0,
            z:0.0
        },0.5 * PI);
    });
    ObjectNode::connect(&engine.genesis,&null_container.objn);

    let mut en1 = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::square(20.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT,
        priority: 0,
    })));


    en1.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(en1)));

    let light = Light::new(ColorRGBu8 {
        r:255,
        g:255,
        b:255
    }.into(), 1.0);
    RefCell::borrow_mut(&light.objn).position.y = 100.0;
    RefCell::borrow_mut(&light.objn).position.z = -100.0;
    engine.lights.push(Rc::new(RefCell::new(light)));

    engine.event_handler.register_update_event(Rc::new(RefCell::new(null_container)));

    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    for i in 0..2048{
        engine.particles.push(CParticle{
            position: Vec3{
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            },
            velocity: Default::default(),
            color: ColorRGBf32{
                r: 1.0,
                g: if i % 2 == 0 { 0.0 } else { 1.0 },
                b: 1.0,
            },
            c: lorem.chars().nth(i % lorem.len()).unwrap(),
            threshold: PI / 400.0,
            mode: ARG
        });
    }

    let rawfdstdin = stdin().as_raw_fd();

    let mut termi = termios::Termios::from_fd(rawfdstdin).unwrap();
    termi.c_lflag &= !termios::os::target::ICANON;
    termi.c_lflag &= !termios::os::target::ECHO;

    termi.c_cc[termios::os::target::VMIN] = 0;
    termi.c_cc[termios::os::target::VTIME] = 0;

    let resultermios:io::Result<()> = termios::tcsetattr(rawfdstdin, termios::os::target::TCSANOW, &mut termi);
    resultermios.unwrap();

    let mut filestdin = unsafe { File::from_raw_fd(rawfdstdin) };

    let mut last_time = Instant::now();

    engine.viewport.clean();
    for _i in 0..65536 {
        let d = 0.5 + f32::sin(_i as f32 * 0.015 * PI) * 0.5;
        for i in 0..2048{
            engine.particles[i].position = d * Vec3{
                x: 1.0 * (i % 80) as f32 - 40.0,
                y: 1.0 * (i / 80) as f32 - 40.0,
                z: 100.0,
            } + (1.0 - d) * Vec3{
                x: 80.0 * f32::cos(i as f32 * 0.1) - 40.0,
                y: 1.0 * i as f32 - 10.0,
                z: 100.0 + 80.0 * f32::sin(i as f32 * 0.1)
            };
        }
        println!("{}",d);
        let mut v = vec![0;1];
        if let Err(e) = filestdin.read_to_end(&mut v){
            println!("{}",e);
            panic!();
        }
        for c in v.iter(){
            let p = cam_objn.borrow_mut().position.clone();
            let d = cam_objn.borrow_mut().direction.clone();
            if *c == b'w'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: 2.0,
                    y: 0.0,
                    z: 0.0,
                });
            }
            else if *c == b's'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: -2.0,
                    y: 0.0,
                    z: 0.0,
                });
            }
            else if *c == b'a'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: 2.0,
                });
            }
            else if *c == b'd'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: -2.0,
                });
            }
            else if *c == b'g'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: 0.0,
                    y: -2.0,
                    z: 0.0,
                });
            }
            else if *c == b'h'{
                cam_objn.borrow_mut().position = p + d.rotate(&Vec3{
                    x: 0.0,
                    y: 2.0,
                    z: 0.0,
                });
            }
            else if *c == b'i'{
                cam_objn.borrow_mut().direction = d * Quaternion::new(Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },PI * 0.02);
            }
            else if *c == b'k'{
                cam_objn.borrow_mut().direction = d * Quaternion::new(Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },PI * -0.02);
            }
            else if *c == b'j'{
                cam_objn.borrow_mut().direction = Quaternion::new(Vec3{
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },PI * -0.02) * d;
            }
            else if *c == b'l'{
                cam_objn.borrow_mut().direction = Quaternion::new(Vec3{
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },PI * 0.02) * d;
            }
        }

        engine.update();
        engine.render();

        let mut dur = last_time.elapsed();
        if dur.as_millis() < 20{
            thread::sleep(Duration::from_millis(20));
            dur = last_time.elapsed();
        }
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll");
        last_time = Instant::now();
    }
}
