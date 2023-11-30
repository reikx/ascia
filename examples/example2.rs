extern crate ascia;

use std::cell::RefCell;
use std::f32::consts::PI;
use std::{env, thread};
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use ascia::ascia::camera::{SimpleCamera};
use ascia::ascia::core::{AsciaEngine, ColorRGBf32, Entity, Material, ObjectNode};
use ascia::ascia::core::MaterialMode::{FLAT, LAMBERT};
use ascia::ascia::lights::Light;
use ascia::ascia::math::{Quaternion, Vec3};
use ascia::ascia::primitives::PrimitiveGenerator;

struct RollingCube{
    size:f32,
    root:Rc<RefCell<ObjectNode>>,
    cube:Rc<RefCell<ObjectNode>>,
    target:Rc<RefCell<ObjectNode>>,
    animation_start:Duration,
    animation_duration:Duration,
    history:VecDeque<(i64,i64)>,
    flashing_floors:Vec<Rc<RefCell<ObjectNode>>>
}

impl RollingCube {
    fn new(size:f32) -> RollingCube{
        let root= ObjectNode::generate();
        let target= ObjectNode::generate();
        let cube = Rc::new(RefCell::new(ObjectNode::from(PrimitiveGenerator::cube(size,Material{
            color: ColorRGBf32{
                r: 0.8,
                g: 0.8,
                b: 0.8,
            },
            mode: LAMBERT,
            priority: 10,
        }))));
        ObjectNode::connect(&root,&cube);
        ObjectNode::connect(&root,&target);
        return RollingCube{
            size:size,
            root: root,
            cube: cube,
            target: target,
            animation_start: Duration::new(0,0),
            animation_duration: Duration::new(0,400000000),
            history:VecDeque::new(),
            flashing_floors: vec![],
        }
    }
}

impl Entity for RollingCube{
    fn repr(&self) -> Rc<RefCell<ObjectNode>> {
        return self.root.clone();
    }

    fn update(&mut self, engine_time: &Duration) {
        if self.history.is_empty(){
            self.history.push_back(((rand::random::<f64>() * 128.0 - 64.0) as i64,(rand::random::<f64>() * 128.0 - 64.0) as i64));
            let last = self.history.back().unwrap();
            if rand::random(){
                if rand::random(){
                    self.history.push_back((last.0 + 1,last.1));
                }
                else{
                    self.history.push_back((last.0 - 1,last.1));
                }
            }
            else{
                if rand::random(){
                    self.history.push_back((last.0,last.1 + 1));
                }
                else{
                    self.history.push_back((last.0,last.1 - 1));
                }
            }
        }
        if *engine_time - self.animation_start >= self.animation_duration{
            let last = self.history.back().unwrap();
            let mut next = (last.0,last.1);
            if rand::random(){
                if rand::random(){
                    next = (last.0 + 1,last.1);
                }
                else{
                    next = (last.0 - 1,last.1);
                }
            }
            else{
                if rand::random(){
                    next = (last.0,last.1 + 1);
                }
                else{
                    next = (last.0,last.1 - 1);
                }
            }
            if (next.0 == self.history[self.history.len() - 2].0) && (next.1 == self.history[self.history.len() - 2].1){
                next.0 = last.0 + (last.0 - self.history[self.history.len() - 2].0);
                next.1 = last.1 + (last.1 - self.history[self.history.len() - 2].1);
            }
            self.history.push_back(next);
            if self.history.len() > 20{
                self.history.pop_front();
            }
            self.animation_start = engine_time.clone();
        }
        else{
            let theta = ((*engine_time - self.animation_start).as_millis() as f32) / (self.animation_duration.as_millis() as f32) * PI * 0.5;
            let l1 = self.history[self.history.len() - 1];
            let l2 = self.history[self.history.len() - 2];
            if l1.1 - l2.1 < 0{
                RefCell::borrow_mut(&self.cube).position = Vec3{
                    x: l2.0 as f32 * self.size,
                    y: 0.0,
                    z: l2.1 as f32 * self.size
                } + Quaternion::new(Vec3{
                    x: 1.0 as f32,
                    y: 0.0,
                    z: 0.0 as f32,
                },-theta).rotate(&Vec3{
                    x: self.size / 2.0,
                    y: self.size / 2.0,
                    z: self.size / 2.0,
                });

                RefCell::borrow_mut(&self.cube).direction = Quaternion::new(Vec3{
                    x: 1.0 as f32,
                    y: 0.0,
                    z: 0.0 as f32,
                },-theta);
            }
            else if l1.1 - l2.1 > 0{
                RefCell::borrow_mut(&self.cube).position = Vec3{
                    x: l1.0 as f32 * self.size,
                    y: 0.0,
                    z: l1.1 as f32 * self.size
                } + Quaternion::new(Vec3{
                    x: 1.0 as f32,
                    y: 0.0,
                    z: 0.0 as f32,
                },-(PI * 0.5 - theta)).rotate(&Vec3{
                    x: self.size / 2.0,
                    y: self.size / 2.0,
                    z: self.size / 2.0,
                });
                RefCell::borrow_mut(&self.cube).direction = Quaternion::new(Vec3{
                    x: 1.0 as f32,
                    y: 0.0,
                    z: 0.0 as f32,
                },-(PI * 0.5 - theta));

            }
            else if l1.0 - l2.0 < 0{
                RefCell::borrow_mut(&self.cube).position = Vec3{
                    x: l2.0 as f32 * self.size,
                    y: 0.0,
                    z: l2.1 as f32 * self.size
                } + Quaternion::new(Vec3{
                    x: 0.0 as f32,
                    y: 0.0,
                    z: 1.0 as f32,
                },theta).rotate(&Vec3{
                    x: self.size / 2.0,
                    y: self.size / 2.0,
                    z: self.size / 2.0,
                });
                RefCell::borrow_mut(&self.cube).direction = Quaternion::new(Vec3{
                    x: 0.0 as f32,
                    y: 0.0,
                    z: 1.0 as f32,
                },theta);
            }
            else{
                RefCell::borrow_mut(&self.cube).position = Vec3{
                    x: l1.0 as f32 * self.size,
                    y: 0.0,
                    z: l1.1 as f32 * self.size
                } + Quaternion::new(Vec3{
                    x: 0.0 as f32,
                    y: 0.0,
                    z: 1.0 as f32,
                },PI * 0.5 - theta).rotate(&Vec3{
                    x: self.size / 2.0,
                    y: self.size / 2.0,
                    z: self.size / 2.0,
                });
                RefCell::borrow_mut(&self.cube).direction = Quaternion::new(Vec3{
                    x: 0.0 as f32,
                    y: 0.0,
                    z: 1.0 as f32,
                },PI * 0.5 - theta);
            }
        }
        self.target.borrow_mut().position.x = self.cube.borrow().position.x;
        self.target.borrow_mut().position.y = self.size;
        self.target.borrow_mut().position.z = self.cube.borrow().position.z;

        self.flashing_floors.resize_with(self.history.len() - 1,|| {
            let rr = Rc::new(RefCell::new(ObjectNode::from(PrimitiveGenerator::square(self.size,Material{
                color: Default::default(),
                mode: FLAT,
                priority: 0,
            }))));
            ObjectNode::connect(&self.root,&rr);
            return rr;
        });
        for i in 0..self.flashing_floors.len(){
            let di = ((*engine_time - self.animation_start).as_millis() as f32) / (self.animation_duration.as_millis() as f32);
            self.flashing_floors[i].borrow_mut().polygons[0].material.color = ColorRGBf32{
                r: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
                g: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
                b: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
            };
            self.flashing_floors[i].borrow_mut().polygons[1].material.color = ColorRGBf32{
                r: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
                g: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
                b: ((i + 1) as f32 - di) / (self.flashing_floors.len() as f32),
            };
            self.flashing_floors[i].borrow_mut().position.x = self.size * (self.history[i].0 as f32 + 0.5);
            self.flashing_floors[i].borrow_mut().position.y = 0.01 * (i + 1) as f32;
            self.flashing_floors[i].borrow_mut().position.z = self.size * (self.history[i].1 as f32 + 0.5);
            self.flashing_floors[i].borrow_mut().direction = Quaternion::new(Vec3{
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },PI * 0.5);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let width:usize = if cfg!(debug_assertions) { 140 } else { usize::from_str(&args[1]).unwrap() };
    let height:usize = if cfg!(debug_assertions) { 40 } else { usize::from_str(&args[2]).unwrap() };
    let mut engine = AsciaEngine::new(width,height);

    let cube_for_camera = Rc::new(RefCell::new(RollingCube::new(10.0)));
    ObjectNode::connect(&engine.genesis,&cube_for_camera.borrow_mut().root);

    let mut camera = SimpleCamera::new(1,3,&engine.wgpu_daq.as_ref().unwrap().0);
    if args.contains(&String::from("--usegpu")) || cfg!(debug_assertions){
        camera.use_gpu = true;
    }
    camera.use_gpu = false;

    let cam_objn = camera.repr().clone();
    cam_objn.borrow_mut().position = Vec3{
        x: -40.0,
        y: 10.0,
        z: -40.0,
    };
    cam_objn.borrow_mut().direction = Quaternion::rotator(&Vec3{
        x: 1.0,
        y: 0.0,
        z: 0.0,
    },&Vec3{
        x: 1.0,
        y: -0.25,
        z: 1.0,
    });
    ObjectNode::connect(&cube_for_camera.borrow_mut().target,&cam_objn);

    engine.event_handler.register_update_event(cube_for_camera);

    {
        let rr = Rc::new(RefCell::new(camera));
        engine.event_handler.register_update_event(rr.clone());
        engine.event_handler.set_render_event(rr);
    }

    for _i in 0..256{
        let cube = RollingCube::new(10.0);
        ObjectNode::connect(&engine.genesis,&cube.root);
        engine.event_handler.register_update_event(Rc::new(RefCell::new(cube)));
    }

    let light = Light::new(ColorRGBf32 {
        r:1.0,
        g:1.0,
        b:1.0
    }, 1.4);
    RefCell::borrow_mut(&light.objn).position = Vec3{
        x: -50.0,
        y: 150.0,
        z: 50.0,
    };
    ObjectNode::connect(&engine.genesis,&light.objn);
    engine.lights.push(Rc::new(RefCell::new(light)));

    let mut last_time = Instant::now();

    engine.viewport.clean();
    for _i in 0..65536 {
        engine.update();
        engine.render();
        let mut dur = last_time.elapsed();
        if dur.as_millis() < 10{
            thread::sleep(Duration::from_millis(5));
            dur = last_time.elapsed();
        }
        println!("fps:{}      ",1000 / dur.as_millis());
        last_time = Instant::now();
    }
}
