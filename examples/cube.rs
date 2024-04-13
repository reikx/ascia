extern crate ascia;

use std::f32::consts::PI;
use std::{env, thread};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ascia::ascia::color::{ColorRGBf32, ColorRGBu8};
use ascia::ascia::core::{AsciaEngine, LambertWithShadowMaterial, ObjectNode, ObjectNodeAttributeDispatcher, PresetAsciaEnvironment, PresetPolygonMaterial, PresetObjectNodeAttributeDispatcher};
use ascia::ascia::lights::PointLight;
use ascia::ascia::math::{Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;
use ascia::ascia::util::{AsciaRenderedFrame, available_preset_cameras, move_camera, preset_camera_info, rotate_camera, TermiosController};

fn main() {
    let args: Vec<String> = env::args().collect();

    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };

    let fps_upper_limit :u64 = 30;

    let mut engine = AsciaEngine::<PresetAsciaEnvironment>::new(width, height);

    let cameras = available_preset_cameras();
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

    let cube = ObjectNode::from("cube 1",PrimitiveGenerator::cube(50.0, PresetPolygonMaterial::LambertWithShadowMaterial(LambertWithShadowMaterial{
        color: ColorRGBf32{
            r: 1.0,
            g: 0.0,
            b: 1.0,
        },
        priority: 0,
    })));

    null_container.add_child(cube);
    engine.genesis_local.add_child(null_container);

    let mut light_objn = ObjectNode::new("light");
    light_objn.position.y = 100.0;
    light_objn.position.z = -100.0;
    light_objn.attribute = PresetObjectNodeAttributeDispatcher::from(PointLight{
        color: ColorRGBu8 {
            r: 255,
            g: 255,
            b: 255
        }.into(),
        power: 1.0,
    }).make_shared();
    engine.genesis_local.add_child(light_objn);

    let capture_flag: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let mut termios_controller = TermiosController::generate(|c, e|{
        if let Some(camera) = e.genesis_local.child_mut("camera"){
            match c {
                b'w' => move_camera(3.0 ,camera, &Vec3{ x: 1.0, y: 0.0, z:0.0 }),
                b's' => move_camera(3.0, camera, &Vec3{ x: -1.0, y: 0.0, z:0.0 }),
                b'a' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: 0.0, z:1.0 }),
                b'd' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: 0.0, z:-1.0 }),
                b'h' => move_camera(3.0, camera,&Vec3{ x: 0.0, y: 1.0, z:0.0 }),
                b'g' => move_camera(3.0, camera, &Vec3{ x: 0.0, y: -1.0, z:0.0 }),
                b'i' => rotate_camera(0.1, camera, &Vec3{ x: 0.0, y: 0.0, z:1.0 }),
                b'k' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: 0.0, z:-1.0 }),
                b'j' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: -1.0, z: 0.0 }),
                b'l' => rotate_camera(0.1,camera, &Vec3{ x: 0.0, y: 1.0, z: 0.0 }),
                b'c' => {
                    *capture_flag.lock().unwrap() = true;
                },
                b'v' => {
                    now_camera_index = (now_camera_index + 1) % cameras.len();
                    camera.attribute = cameras[now_camera_index].clone()
                },
                _ => {}
            }
        }
    }).unwrap();

    let mut last_time = Instant::now();

    loop{
        engine.sync_engine_time();

        termios_controller.input(&mut engine).expect("something went wrong with processing input from keyboard");
        engine.update_global_nodes();

        if let Ok(data) = engine.render(engine.genesis_global.child("camera").unwrap()){
            let mut mg = capture_flag.lock().unwrap();
            if cfg!(feature = "export") && *mg{
                let _ = AsciaRenderedFrame::generate(&data, &engine.engine_time()).unwrap().save(&format!("./cube_{}.json", engine.engine_time().as_millis()));
                *mg = false;
            }
        }

        if (last_time.elapsed().as_millis() as u64) < (1000 / fps_upper_limit){
            thread::sleep(Duration::from_millis(1000 / fps_upper_limit - last_time.elapsed().as_millis() as u64));
        }

        let dur = last_time.elapsed();
        println!("dur:{}      ",dur.as_millis());
        println!("fps:{}      ",1000 / dur.as_millis());
        println!("press [W][A][S][D] to move horizontally, [G][H] to move vertically, [I][J][K][L] to roll, [V] to change camera");
        println!("current camera: {}     ", preset_camera_info(&engine.genesis_global.child("camera").unwrap().attribute));

        last_time = Instant::now();
    }
}
