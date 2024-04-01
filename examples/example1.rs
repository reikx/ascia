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
use ascia::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use ascia::ascia::camera_gpu::GPUWrapper;
use ascia::ascia::color::{ColorRGBf32, ColorRGBu8};
use ascia::ascia::core::{AsciaEngine, FlatMaterial, LambertMaterial, LambertWithShadowMaterial, Local, Material, ObjectNode, PresetCamera, PresetLight, PresetMaterial, ObjectNodeAttribute, PresetObjectNodeAttributeDispatcher, ObjectNodeAttributeDispatcher, PresetAsciaEnvironment};
use ascia::ascia::lights::{PointLight};
use ascia::ascia::math::{Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    
    let mut engine = AsciaEngine::new(width,height);

    let cameras = available_cameras(&engine);
    let mut now_camera_index = 0usize;

    let mut cam_objn = ObjectNode::new("camera");
    cam_objn.attribute = cameras[now_camera_index].clone();
    cam_objn.direction = Quaternion::new(&Vec3{
        x: 0.0,
        y: 1.0,
        z: 0.0,
    }, -PI * 0.5, 1.0);
    
    engine.genesis_local.add_child(cam_objn);
    
    let mut null_container = ObjectNode::new("null container");
    null_container.position = Vec3{
        x:0.0,
        y:0.0,
        z:25.0
    };
    
    let mut cube_1 = ObjectNode::from("cube 1", PrimitiveGenerator::cube(6.0,PresetMaterial::LambertMaterial(LambertMaterial{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        priority:0
    })));
    cube_1.position = Vec3{
        x:-6.0,
        y:0.0,
        z:0.0
    };
    
    null_container.add_child(cube_1);

    let mut cube_2 = ObjectNode::from("cube 2", PrimitiveGenerator::cube(6.0,PresetMaterial::LambertMaterial(LambertMaterial{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        priority:0
    })));
    cube_2.position = Vec3{
        x:6.0,
        y:0.0,
        z:0.0
    };
    null_container.add_child(cube_2);
    
    let mut cube_3 = ObjectNode::from("cube 3", PrimitiveGenerator::cube(6.0,PresetMaterial::LambertMaterial(LambertMaterial{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        priority:0
    })));

    cube_3.position = Vec3{
        x:8.0,
        y:0.0,
        z:14.0
    };
    null_container.add_child(cube_3);

    let mut cube_4 = ObjectNode::from("cube 3", PrimitiveGenerator::cube(6.0,PresetMaterial::LambertMaterial(LambertMaterial{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        priority:0
    })));
    
    cube_4.position = Vec3{
        x:0.0,
        y:0.0,
        z:20.0
    };
    null_container.add_child(cube_4);
    
    let mut red_square = ObjectNode::from("red square", PrimitiveGenerator::square(20.0,PresetMaterial::LambertWithShadowMaterial(
        LambertWithShadowMaterial{
            color: ColorRGBf32{
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            priority: 1,
        }
    )));
    
    null_container.add_child(red_square);

    let mut green_square = ObjectNode::from("green square", PrimitiveGenerator::square(20.0,PresetMaterial::FlatMaterial(
        FlatMaterial{
            color: ColorRGBf32{
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            priority: 0,
        }
    )));

    green_square.position.z = 30.0;
    green_square.direction = Quaternion::new(&Vec3{
        x:0.0,
        y:1.0,
        z:0.0
    },-PI * 0.5, 1.0);
    null_container.add_child(green_square);

    let mut blue_square = ObjectNode::from("blue square", PrimitiveGenerator::square(20.0,PresetMaterial::LambertWithShadowMaterial(
        LambertWithShadowMaterial{
            color: ColorRGBf32{
                r: 0.0,
                g: 0.0,
                b: 1.0,
            },
            priority: 0,
        }
    )));
    null_container.add_child(blue_square);
    
    for i in 0..64{
        let mut tiny_cube = ObjectNode::from(&format!("tiny cube {}", i), PrimitiveGenerator::cube(10.0,PresetMaterial::LambertWithShadowMaterial(
            LambertWithShadowMaterial{
                color: ColorRGBu8{
                    r: ((16i32 * (32i32 - i)) % 255i32) as u8,
                    g: 128,
                    b: ((16 * i) % 255) as u8,
                }.into(),
                priority: 0,
            }
        )));
        tiny_cube.position = Quaternion::new(&Vec3{
            x:0.0,
            y:1.0,
            z:0.0
        }.normalize(),(i as f32) * 0.3, 1.0).rotate(&Vec3{
            x:60.0,
            y:0.0,
            z:0.0
        }).add(Vec3{
            x:0.0,
            y:i as f32 * 10.0,
            z:0.0
        });
        null_container.add_child(tiny_cube);
    }
    
    let pointlight = PresetLight::PointLight(PointLight{
        color: ColorRGBf32{
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
        power: 1.0,
    });

    let mut light = ObjectNode::new("light");
    light.attribute = PresetObjectNodeAttributeDispatcher::Light(pointlight).make_shared();
    light.position.x = 30.0;
    light.position.y = 30.0;
    light.position.z = -30.0;
    engine.genesis_local.add_child(light);
    
    engine.genesis_local.add_child(null_container);

    let mut input = create_stdin_controller().unwrap();
    
    let mut last_time = Instant::now();

    for _i in 0..65536 {
        engine.sync_engine_time();
        let d = engine.engine_time();

        {
            let mut null_container = engine.genesis_local.child_mut("null container").unwrap();
            null_container.direction = Quaternion::new(&Vec3{
                x:1.0,
                y:0.0,
                z:0.0
            },1.0, 1.0) * Quaternion::new(&Vec3{
                x:0.0,
                y:1.0,
                z:0.0
            },(d.as_millis() as f32) / 5000.0 * 2.0 * PI, 1.0);
        }
        {
            let mut cube_1 = engine.genesis_local.child_mut("null container").unwrap().child_mut("cube 1").unwrap();
            cube_1.position.y = 8.0 * f32::sin((d.as_millis() as f32) / 2000.0 * 2.0 * PI);
            cube_1.direction = Quaternion::new(&Vec3{
                x:1.0,
                y:1.0,
                z:1.0
            }.normalize(),(d.as_millis() as f32) / 2000.0 * 2.0 * PI, 1.0);
        }
        {
            let mut cube_2 = engine.genesis_local.child_mut("null container").unwrap().child_mut("cube 1").unwrap();
            cube_2.position.y = 3.0 * f32::cos((d.as_millis() as f32) / 3000.0 * 2.0 * PI);
            cube_2.direction = Quaternion::new(&Vec3{
                x:1.0,
                y:0.0,
                z:1.0
            }.normalize(),(d.as_millis() as f32) / 3000.0 * 2.0 * PI, 1.0);
        }
        {
            let mut blue_square = engine.genesis_local.child_mut("null container").unwrap().child_mut("blue square").unwrap();
            blue_square.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },(d.as_millis() as f32) / 2500.0 * 2.0 * PI, 1.0);
        }
        
        move_camera(engine.genesis_local.child_mut("camera").unwrap(),&mut input,3.0,0.1, &cameras, &mut now_camera_index);
        engine.update_global_nodes();
        engine.render(engine.genesis_global.child("camera").unwrap());

        while last_time.elapsed().as_millis() < 10{
            thread::sleep(Duration::from_millis(2));
        }
        let mut dur = last_time.elapsed();

        println!("dur:{}      ",dur.as_millis());
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", camera_info(&cameras[now_camera_index]));
        
        last_time = Instant::now();
    }
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
fn available_cameras(engine: &AsciaEngine<PresetAsciaEnvironment>) -> Vec<Rc<RefCell<Option<PresetObjectNodeAttributeDispatcher<PresetAsciaEnvironment>>>>>{
    let aov = (PI / 3.0, PI / 4.0);

    let mut cameras = vec![
        SimpleCamera::<PresetAsciaEnvironment>::new(aov, 1).make_attribute_enum().make_shared(),
        SimpleBVHCamera::<PresetAsciaEnvironment>::new(aov, 1).make_attribute_enum().make_shared(),
        SimpleCamera::<PresetAsciaEnvironment>::new(aov, 3).make_attribute_enum().make_shared(),
        SimpleBVHCamera::<PresetAsciaEnvironment>::new(aov, 3).make_attribute_enum().make_shared(),
    ];

    if engine.wgpu_daq.is_some(){
        cameras.push(GPUWrapper::<PresetAsciaEnvironment, SimpleCamera<PresetAsciaEnvironment>>::generate(SimpleCamera::new(aov, 1), &engine).make_attribute_enum().make_shared());
        cameras.push(GPUWrapper::<PresetAsciaEnvironment, SimpleBVHCamera<PresetAsciaEnvironment>>::generate(SimpleBVHCamera::new(aov, 1), &engine).make_attribute_enum().make_shared());
        cameras.push(GPUWrapper::<PresetAsciaEnvironment, SimpleCamera<PresetAsciaEnvironment>>::generate(SimpleCamera::new(aov, 3), &engine).make_attribute_enum().make_shared());
        cameras.push(GPUWrapper::<PresetAsciaEnvironment, SimpleBVHCamera<PresetAsciaEnvironment>>::generate(SimpleBVHCamera::new(aov, 3), &engine).make_attribute_enum().make_shared());
    }
    return cameras;
}

fn move_camera(mut cam_objn: &mut ObjectNode<PresetAsciaEnvironment, Local>, input: &mut File, velocity:f32, rotation_speed:f32, cameras: &Vec<Rc<RefCell<Option<PresetObjectNodeAttributeDispatcher<PresetAsciaEnvironment>>>>>, mut now_camera_index: &mut usize){
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
            cam_objn.direction = d * Quaternion::new(&Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },rotation_speed, 1.0);
        }
        else if *c == b'k'{
            cam_objn.direction = d * Quaternion::new(&Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },-rotation_speed, 1.0);
        }
        else if *c == b'j'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },-rotation_speed, 1.0) * d;
        }
        else if *c == b'l'{
            cam_objn.direction = Quaternion::new(&Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },rotation_speed, 1.0) * d;
        }
        else if *c == b'v'{
            *now_camera_index = if *now_camera_index + 1 >= cameras.len() { 0 } else { *now_camera_index + 1 };
            cam_objn.attribute = cameras[*now_camera_index].clone();
        }
    }
}

fn camera_info(attr: &Rc<RefCell<Option<PresetObjectNodeAttributeDispatcher<PresetAsciaEnvironment>>>>) -> String{
    if let Some(PresetObjectNodeAttributeDispatcher::Camera(c)) = &*RefCell::borrow(attr){
        return match c {
            PresetCamera::SimpleCamera(cam) => { format!("simple cpu {}x", cam.sampling_size) }
            PresetCamera::SimpleCameraGPU(cam) => { format!("simple gpu {}x", cam.cpu_camera.sampling_size) }
            PresetCamera::SimpleBVHCamera(cam) => { format!("simple cpu bvh {}x", cam.sampling_size) }
            PresetCamera::SimpleBVHCameraGPU(cam) => { format!("simple gpu bvh {}x", cam.cpu_camera.sampling_size)}
        };
    }
    return "unknown".to_string();
}

