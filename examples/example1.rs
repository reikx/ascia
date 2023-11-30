extern crate ascia;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::fs::File;
use std::{env, io, thread};
use std::io::{Read, stdin};
use std::ops::Add;
use std::os::fd::{AsRawFd, FromRawFd};
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use ascia::ascia::camera::{SimpleCamera};
use ascia::ascia::core::{AsciaEngine, ColorRGBf32, ColorRGBu8, Entity, Material, ObjectNode, SimpleEntity};
use ascia::ascia::core::MaterialMode::{FLAT, LAMBERT, LAMBERT_WITH_SHADOW};
use ascia::ascia::lights::Light;
use ascia::ascia::math::{Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let mut camera = SimpleCamera::new(1,3,&engine.wgpu_daq.as_ref().unwrap().0);
    if args.contains(&String::from("--usegpu")) || cfg!(debug_assertions){
        camera.use_gpu = true;
    }

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
        z:25.0
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
    ObjectNode::connect(&engine.genesis,&null_container.objn);

    let mut cu1 = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(6.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT,
        priority:0,
    })));

    cu1.objn.borrow_mut().position = Vec3{
        x:-6.0,
        y:0.0,
        z:0.0
    };
    cu1.register_update_fn(|mut objn,d|{
        objn.position.y = 8.0 * f32::sin((d.as_millis() as f32) / 2000.0 * 2.0 * PI);
        objn.direction = Quaternion::new(Vec3{
            x:1.0,
            y:1.0,
            z:1.0
        }.normalize(),(d.as_millis() as f32) / 2000.0 * 2.0 * PI);
    });
    cu1.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(cu1)));

    let mut cu2 = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(6.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT,
        priority: 0,
    })));
    cu2.objn_refmut().position = Vec3{
        x:6.0,
        y:0.0,
        z:0.0
    };
    cu2.register_update_fn(|mut objn,d|{
        objn.position.y = 3.0 * f32::cos((d.as_millis() as f32) / 3000.0 * 2.0 * PI);
        objn.direction = Quaternion::new(Vec3{
            x:1.0,
            y:0.0,
            z:1.0
        }.normalize(),(d.as_millis() as f32) / 3000.0 * 2.0 * PI);
    });
    cu2.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(cu2)));

    let cu3 = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(6.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT,
        priority: 0,
    })));
    cu3.objn_refmut().position = Vec3{
        x:8.0,
        y:0.0,
        z:14.0
    };
    cu3.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(cu3)));

    let cu4 = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(6.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        mode: LAMBERT,
        priority: 0,
    })));
    cu4.objn_refmut().position = Vec3{
        x:0.0,
        y:0.0,
        z:20.0
    };
    cu4.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(cu4)));


    let red_sq = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::square(20.0,Material{
        color: ColorRGBf32{
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        mode: LAMBERT_WITH_SHADOW,
        priority: 1,
    })));
    red_sq.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(red_sq)));


    let green_sq = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::square(20.0,Material{
        color: ColorRGBf32{
            r: 0.0,
            g: 1.0,
            b: 0.0,
        },
        mode: FLAT,
        priority: 0,
    })));
    green_sq.objn_refmut().position.z = 30.0;
    green_sq.objn_refmut().direction = Quaternion::new(Vec3{
        x:0.0,
        y:1.0,
        z:0.0
    },-PI * 0.5);
    green_sq.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(green_sq)));


    let mut blue_sq = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::square(20.0,Material{
        color: ColorRGBf32{
            r: 0.0,
            g: 0.0,
            b: 1.0,
        },
        mode: LAMBERT_WITH_SHADOW,
        priority: 0,
    })));
    blue_sq.register_update_fn(|mut objn,d|{
        objn.direction = Quaternion::new(Vec3{
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },(d.as_millis() as f32) / 2500.0 * 2.0 * PI);
    });
    blue_sq.connect(&null_container);
    engine.event_handler.register_update_event(Rc::new(RefCell::new(blue_sq)));



    for i in 0..64{
        let tiny_cube = SimpleEntity::new(ObjectNode::from(PrimitiveGenerator::cube(10.0,Material{
            color: ColorRGBu8{
                r: ((16i32 * (32i32 - i)) % 255i32) as u8,
                g: 128,
                b: ((16 * i) % 255) as u8,
            }.into(),
            mode: LAMBERT_WITH_SHADOW,
            priority: 0,
        })));
        tiny_cube.objn_refmut().position = Quaternion::new(Vec3{
            x:0.0,
            y:1.0,
            z:0.0
        }.normalize(),(i as f32) * 0.3).rotate(&Vec3{
            x:60.0,
            y:0.0,
            z:0.0
        }).add(Vec3{
            x:0.0,
            y:i as f32 * 10.0,
            z:0.0
        });
        tiny_cube.connect(&null_container);
        engine.event_handler.register_update_event(Rc::new(RefCell::new(tiny_cube)));

    }

    let light = Light::new(ColorRGBu8 {
        r:255,
        g:255,
        b:255
    }.into(), 1.0);
    RefCell::borrow_mut(&light.objn).position.y = 20.0;
    RefCell::borrow_mut(&light.objn).position.z = 0.0;
    engine.lights.push(Rc::new(RefCell::new(light)));

    engine.event_handler.register_update_event(Rc::new(RefCell::new(null_container)));

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
                cam_objn.borrow_mut().direction = Quaternion::new(d.rotate(&Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }),PI * 0.02) * d;
            }
            else if *c == b'k'{
                cam_objn.borrow_mut().direction = Quaternion::new(d.rotate(&Vec3{
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }),PI * -0.02) * d;
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
