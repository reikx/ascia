use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::min;
use std::f32::consts::PI;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, ComputePipeline, Device, ShaderModule, ShaderModuleDescriptor};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::ascia::core::{AsciaEngine, CameraEntity, ColorRGBu8, CParticle, Entity, Material, ObjectNode, Polygon, Ray, RenderChar};
use crate::ascia::math::{Matrix33, Vec3};
use crate::ascia::raycaster::{Raycaster, RaycasterIntersection};
use crate::ascia::charmapper::CharMapper3x3;
use crate::ascia::core::CParticleMode::SPHERE;
use crate::ascia::lights::Light;

pub struct SimpleCamera{
    cpu_camera:SimpleCPUCamera,
    gpu_camera:SimpleGPUCamera,
    root:Rc<RefCell<ObjectNode>>,
    pub use_gpu:bool
}

impl SimpleCamera {
    pub fn new(sampling_size:usize,particle_rerender_num:usize,device:&Device) -> SimpleCamera{
        let cc = SimpleCPUCamera::new(sampling_size, particle_rerender_num);
        let gc = SimpleGPUCamera::new(sampling_size, particle_rerender_num,device);
        let root = ObjectNode::generate();
        ObjectNode::connect(&root,&cc.repr());
        ObjectNode::connect(&root,&gc.repr());
        return SimpleCamera{
            cpu_camera: cc,
            gpu_camera: gc,
            root:root,
            use_gpu: false,
        }
    }
    pub fn set_sampling_size(&mut self,ss:usize){
        self.gpu_camera.sampling_size = ss;
        self.cpu_camera.sampling_size = ss;
    }
}

impl Entity for SimpleCamera {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }

    fn update(&mut self, engine_time: &Duration) {
        if engine_time.as_secs() % 4 < 4{
            self.use_gpu = true;
        }
        else{
            self.use_gpu = false;
        }

        if self.use_gpu{
            self.set_sampling_size(3);
        }
        else{
            self.set_sampling_size(3);
        }

        if self.use_gpu { self.gpu_camera.update(engine_time)} else {self.cpu_camera.update(engine_time)};
    }
}

impl CameraEntity for SimpleCamera{
    fn angle_of_view(&self) -> (f32, f32) {
        return if self.use_gpu { self.gpu_camera.angle_of_view()} else { self.cpu_camera.angle_of_view()};
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        return if self.use_gpu { self.gpu_camera.render(engine)} else { self.cpu_camera.render(engine)};
    }
}

pub struct SimpleBVHCamera{
    cpu_camera:BVHCPUCamera,
    gpu_camera:BVHGPUCamera,
    root:Rc<RefCell<ObjectNode>>,
    pub use_gpu:bool
}

impl SimpleBVHCamera {
    pub fn new(sampling_size:usize,particle_rerender_num:usize,device:&Device) -> SimpleBVHCamera{
        let cc = BVHCPUCamera::new(sampling_size,particle_rerender_num);
        let gc = BVHGPUCamera::new(sampling_size,particle_rerender_num,device);
        let root = ObjectNode::generate();
        ObjectNode::connect(&root,&cc.repr());
        ObjectNode::connect(&root,&gc.repr());
        return SimpleBVHCamera{
            cpu_camera: cc,
            gpu_camera: gc,
            root:root,
            use_gpu: false,
        }
    }
    pub fn set_sampling_size(&mut self,ss:usize){
        self.gpu_camera.sampling_size = ss;
        self.cpu_camera.sampling_size = ss;
    }
}


impl Entity for SimpleBVHCamera {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }

    fn update(&mut self, engine_time: &Duration) {
        if self.use_gpu { self.gpu_camera.update(engine_time)} else {self.cpu_camera.update(engine_time)};
    }
}

impl CameraEntity for SimpleBVHCamera{
    fn angle_of_view(&self) -> (f32, f32) {
        return if self.use_gpu { self.gpu_camera.angle_of_view()} else { self.cpu_camera.angle_of_view()};
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        return if self.use_gpu { self.gpu_camera.render(engine)} else { self.cpu_camera.render(engine)};
    }
}

pub struct SimpleCPUCamera {
    pub root:Rc<RefCell<ObjectNode>>,
    pub sampling_size:usize,
    pub particle_rerender_num:usize,
    cm: CharMapper3x3
}

impl SimpleCPUCamera {
    pub fn new(sampling_size:usize,particle_rerender_num:usize) -> SimpleCPUCamera{
        return SimpleCPUCamera{
            root: ObjectNode::generate(),
            sampling_size,
            particle_rerender_num,
            cm: CharMapper3x3::new()
        }
    }
}

impl Entity for SimpleCPUCamera {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }

    fn update(&mut self,_d:&Duration) {

    }
}

impl SimpleCPUCamera{
    fn render_1x(&self,engine: &AsciaEngine) -> Vec<Vec<RenderChar>>{
        let height = engine.viewport.height();
        let width = engine.viewport.width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];
        let global_polygons:Vec<Polygon> = engine.genesis.borrow_mut().global_polygons_recursive();
        let mut results:Vec<Vec<Option<RaycasterIntersection>>> = vec![vec![Option::None;height];width];

        for y in 0..height{
            for x in 0..width{
                let mut result:Option<RaycasterIntersection> = None;
                for k in 0..global_polygons.len(){
                    if let Some(r) = Raycaster::project_polygon(&self.global_position(), &self.global_direction().rotate(&Vec3{
                        x:1.0,
                        y:f32::tan(self.angle_of_view().1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                        z:f32::tan(self.angle_of_view().0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                    }), k, &global_polygons){
                        if let Some(rn) = result{
                            if r.depth < rn.depth{
                                result = Some(r);
                            }
                        }
                        else{
                            result = Some(r);
                        }
                    }
                }
                results[x][y] = result;
            }
        }

        for y in 0..height {
            for x in 0..width {
                if let Some(result) = results[x][y]{
                    out[y][x].c = '#';
                    out[y][x].color = global_polygons[result.polygon_index].material.shade(&global_polygons,&Ray{
                        position: self.global_position(),
                        direction: result.depth * Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view().1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view().0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        }.normalize(),
                    }, result.polygon_index, result.depth as f32, &result.intersection_position_on_polygon, &result.intersection_position_global, &self.global_position(), &engine.lights).into();
                }
            }
        }
        return out;
    }

    fn render_3x(&self,engine: &AsciaEngine) -> Vec<Vec<RenderChar>>{
        let height = engine.viewport.height();
        let width = engine.viewport.width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];

        let global_polygons:Vec<Polygon> = engine.genesis.borrow_mut().global_polygons_recursive();

        let mut results:Vec<Vec<Option<RaycasterIntersection>>> = vec![vec![Option::None;height * self.sampling_size]; width * self.sampling_size];

        for y in 0..height{
            for x in 0..width{
                for i in 0..self.sampling_size {
                    for j in 0..self.sampling_size {
                        let mut result:Option<RaycasterIntersection> = None;
                        for k in 0..global_polygons.len(){
                            if let Some(r) = Raycaster::project_polygon(&self.global_position(), &self.global_direction().rotate(&Vec3{
                                x:1.0,
                                y:f32::tan(self.angle_of_view().1 * 0.5) * (1.0 - 2.0 * (y * self.sampling_size + i) as f32 / (height * self.sampling_size) as f32),
                                z:f32::tan(self.angle_of_view().0 * 0.5) * (1.0 - 2.0 * (x * self.sampling_size + j) as f32 / (width * self.sampling_size) as f32),
                            }), k, &global_polygons){
                                if let Some(rn) = result{
                                    if r.depth < rn.depth{
                                        result = Some(r);
                                    }
                                }
                                else{
                                    result = Some(r);
                                }
                            }
                        }
                        results[x * self.sampling_size + j][y * self.sampling_size + i] = result;
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let mut max_priority:u32 = 0;
                for i in 0..self.sampling_size {
                    for j in 0..self.sampling_size {
                        if let Some(r) = results[x * self.sampling_size + j][y * self.sampling_size + i]{
                            if max_priority < global_polygons[r.polygon_index].material.priority{
                                max_priority = global_polygons[r.polygon_index].material.priority;
                            }
                        }
                    }
                }

                let mut r:u32 = 0;
                let mut g:u32 = 0;
                let mut b:u32 = 0;
                let mut count:u32 = 0;
                let mut seg:usize = 0;

                for i in 0..self.sampling_size{
                    for j in 0..self.sampling_size{
                        if i == 1 && j == 1{
                            continue;
                        }
                        seg <<= 1;

                        if let Some(result) = results[x * self.sampling_size + j][y * self.sampling_size + i]{
                            if max_priority == global_polygons[result.polygon_index].material.priority{
                                seg |= 1;
                                let c:ColorRGBu8 = global_polygons[result.polygon_index].material.shade(&global_polygons,&Ray{
                                    position: self.global_position(),
                                    direction: result.depth * Vec3{
                                        x:1.0,
                                        y:f32::tan(self.angle_of_view().1 * 0.5) * (1.0 - 2.0 * (y * self.sampling_size + i) as f32 / (height * self.sampling_size) as f32),
                                        z:f32::tan(self.angle_of_view().0 * 0.5) * (1.0 - 2.0 * (x * self.sampling_size + j) as f32 / (width * self.sampling_size) as f32),
                                    }.normalize(),
                                }, result.polygon_index, result.depth as f32, &result.intersection_position_on_polygon, &result.intersection_position_global, &self.global_position(), &engine.lights).into();
                                r += c.r as u32;
                                g += c.g as u32;
                                b += c.b as u32;
                                count += 1;
                            }
                        }
                    }
                }

                if self.sampling_size == 1{
                    out[y][x].c = '#';
                }
                else if self.sampling_size == 3{
                    out[y][x].c = self.cm.mem[seg];
                }
                else{
                    panic!();
                }
                out[y][x].color = if count == 0 { ColorRGBu8::default()} else { ColorRGBu8{
                    r: if r / count <= u8::MAX as u32 { (r / count) as u8} else {u8::MAX},
                    g: if g / count <= u8::MAX as u32 { (g / count) as u8} else {u8::MAX},
                    b: if b / count <= u8::MAX as u32 { (b / count) as u8} else {u8::MAX},
                }};
            }
        }

        return out;
    }
}

impl CameraEntity for SimpleCPUCamera {
    fn angle_of_view(&self) -> (f32,f32) {
        return (PI / 3.0,PI / 4.0);
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>>{
        if self.sampling_size == 1{
            return self.render_1x(engine);
        }
        else if self.sampling_size == 3{
            return self.render_3x(engine);
        }
        else{
            return vec![vec![RenderChar::default();engine.viewport.height()];engine.viewport.width()];;
        }
    }
}

pub struct SimpleGPUCamera {
    pub root:Rc<RefCell<ObjectNode>>,
    sampling_size:usize,
    particle_rerender_num:usize,
    cm: CharMapper3x3,
    shader: ShaderModule,
    render_pipeline_1x: ComputePipeline,
    render_pipeline_3x: ComputePipeline,
    render_cp_pipeline: ComputePipeline,
}

impl SimpleGPUCamera{
    pub fn new(sampling_size:usize,particle_rerender_num:usize,device:&Device) -> SimpleGPUCamera{
        let shader = device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: (wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simple_raytracing_renderer.wgsl")))),
        });
        return SimpleGPUCamera{
            root:ObjectNode::generate(),
            sampling_size:sampling_size,
            cm: CharMapper3x3::new(),
            particle_rerender_num,
            render_pipeline_1x: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_1x",
            }),
            render_pipeline_3x: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_3x",
            }),
            render_cp_pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_cp",
            }),
            shader:shader,
        }
    }
}

impl Entity for SimpleGPUCamera {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }

    fn update(&mut self, _engine_time: &Duration) {

    }
}

impl CameraEntity for SimpleGPUCamera {
    fn angle_of_view(&self) -> (f32,f32) {
        return (PI / 3.0,PI / 4.0);
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        let height = engine.viewport.height();
        let width = engine.viewport.width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];

        let mut settings_b:Vec<u8> = vec![];
        let gp = self.global_position();
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.x as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.y as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.z as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as f32)));

        let gd = self.global_direction().vec4;
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.w as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.x as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.y as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.z as f32)));

        settings_b.extend_from_slice(bytemuck::bytes_of(&(width as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(height as u32)));

        settings_b.extend_from_slice(bytemuck::bytes_of(&self.angle_of_view().0));
        settings_b.extend_from_slice(bytemuck::bytes_of(&self.angle_of_view().1));

        settings_b.extend_from_slice(bytemuck::bytes_of(&(3 as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(self.sampling_size as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as u32)));


        let global_polygons = &engine.genesis.borrow_mut().global_polygons_recursive();

        let lights = &engine.lights;
        let mut lights_b:Vec<u8> = vec![0;32 * lights.len()];
        for i in 0..lights.len(){
            write_light_into_u8s(&mut lights_b[(i * 32)..((i + 1) * 32)],&RefCell::borrow(&lights[i]));
        }

        let particles = &engine.particles;
        let mut particles_b = vec![0;if particles.len() != 0 {particles.len() * 64} else { 64 }];
        for i in 0..particles.len(){
            write_particle_into_u8s(&mut particles_b[(i * 64)..((i + 1) * 64)],&particles[i]);
        }

        // insert a phantom particle in order to avoid shader error
        if particles.len() == 0{
            write_particle_into_u8s(&mut particles_b[0..64],&CParticle {
                position: Default::default(),
                velocity: Default::default(),
                color: Default::default(),
                c: ' ',
                threshold: 0.0,
                mode: SPHERE,
            });
        }

        let mut char_mapper_b:Vec<u8> = vec![0;4 * 256];
        for i in 0..256{
            char_mapper_b[(i * 4)..(i * 4 + 4)].copy_from_slice(bytemuck::bytes_of(&(self.cm.mem[i] as u32)));
        }

        if let Some((device,queue)) = &engine.wgpu_daq{
            let settings_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: settings_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let mut polygons_u8s = vec![0;(80 * global_polygons.len())];
            for i in 0..global_polygons.len(){
                write_polygon_into_u8s(&mut polygons_u8s[(i * 80)..((i + 1) * 80)],&global_polygons[i]);
            }

            let polygons_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: polygons_u8s.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let lights_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: lights_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let particles_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: particles_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let cp_counter_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: if particles.len() != 0 {particles.len() * 4} else { 4 } as u64,
                usage: wgpu::BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let char_mapper_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: char_mapper_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let dots_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height * if self.sampling_size == 1 {1} else {8}) * 12) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let cp_results_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 12) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let chars_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 4) as u64,
                    usage: (BufferUsages::STORAGE | BufferUsages::COPY_SRC),
                    mapped_at_creation: false,
                }
            );

            let chars_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 4) as u64,
                    usage: (wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                }
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some("ASCIA") });

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &self.render_cp_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 6,
                            resource: particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 7,
                            resource: cp_counter_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &self.render_cp_pipeline.get_bind_group_layout(1) ,
                    entries: &[
                        BindGroupEntry{
                            binding: 2,
                            resource: cp_results_buffer_storage.as_entire_binding(),
                        },
                    ],
                });

                for _ in 0..self.particle_rerender_num{
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER_CP") });
                    cpass.set_pipeline(&self.render_cp_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.insert_debug_marker("awww");
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            let render_pipeline = if self.sampling_size == 3 { &self.render_pipeline_3x } else {&self.render_pipeline_1x};
            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &render_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: lights_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: char_mapper_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 6,
                            resource: particles_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &render_pipeline.get_bind_group_layout(1) ,
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: dots_buffer_storage.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: chars_buffer_storage.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: cp_results_buffer_storage.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(render_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.insert_debug_marker("awww");
                    if self.sampling_size == 1{
                        cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                    }
                    else{
                        cpass.dispatch_workgroups((width / 4 + 1) as u32,(height / 2 + 1) as u32,1);
                    }
                }
            }

            encoder.copy_buffer_to_buffer(&chars_buffer_storage,0,&chars_buffer_staging,0,chars_buffer_storage.size());

            queue.submit(Some(encoder.finish()));

            let chars_results_slice = chars_buffer_staging.slice(..);
            let (sender,receiver) = futures_intrusive::channel::shared::oneshot_channel();
            chars_results_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender.send(v).unwrap()
            });

            device.poll(wgpu::Maintain::Wait);

            if let Some(Ok(())) = pollster::block_on(receiver.receive()){
                let s = chars_results_slice.get_mapped_range();
                let results:Vec<u32> = bytemuck::cast_slice(&s).to_vec();
                for x in 0..width{
                    for y in 0..height{
                        let result = results[y * width + x];
                        out[y][x] = RenderChar{
                            c: ((result & 0xff) as u8) as char,
                            color: ColorRGBu8 {
                                r: ((result >> 24) & 0xff) as u8,
                                g: ((result >> 16) & 0xff) as u8,
                                b: ((result >> 8) & 0xff) as u8,
                            },
                        }
                    }
                }
            }
        }
        return out;
    }
}



pub struct BVHCPUCamera{
    pub root:Rc<RefCell<ObjectNode>>,
    sampling_size:usize,
    particle_rerender_num:usize,
    cm: CharMapper3x3
}

impl BVHCPUCamera {
    pub fn new(sampling_size:usize,particle_rerender_num:usize) -> BVHCPUCamera{
        return BVHCPUCamera{
            root: ObjectNode::generate(),
            sampling_size,
            particle_rerender_num,
            cm: CharMapper3x3::new(),
        }
    }
}

impl Entity for BVHCPUCamera{
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }
    fn update(&mut self, engine_time: &Duration) {

    }
}

impl CameraEntity for BVHCPUCamera{
    fn angle_of_view(&self) -> (f32,f32) {
        return (PI / 3.0,PI / 4.0);
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        todo!()
    }
}

pub struct BVHGPUCamera {
    pub root:Rc<RefCell<ObjectNode>>,
    sampling_size:usize,
    particle_rerender_num:usize,
    cm: CharMapper3x3,
    shader: ShaderModule,
    init_bvh_pipeline: ComputePipeline,
    render_pipeline_1x: ComputePipeline,
    render_pipeline_3x: ComputePipeline,
    render_cp_pipeline: ComputePipeline,
}

impl BVHGPUCamera {
    pub fn new(sampling_size:usize,particle_rerender_num:usize,device:&Device) -> BVHGPUCamera {
        let shader = device.create_shader_module(ShaderModuleDescriptor{
            label: None,
            source: (wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simple_raytracing_renderer_bvh.wgsl")))),
        });
        return BVHGPUCamera {
            root:ObjectNode::generate(),
            sampling_size:sampling_size,
            cm: CharMapper3x3::new(),
            particle_rerender_num,
            init_bvh_pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "init_bvh",
            }),
            render_pipeline_1x: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_1x",
            }),
            render_pipeline_3x: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_3x",
            }),
            render_cp_pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader,
                entry_point: "render_cp",
            }),
            shader:shader,
        }
    }
}

impl Entity for BVHGPUCamera {
    fn repr<'a>(&'a self) -> &'a Rc<RefCell<ObjectNode>> {
        return &self.root;
    }
    fn update(&mut self, _engine_time: &Duration) {

    }
}

impl CameraEntity for BVHGPUCamera {
    fn angle_of_view(&self) -> (f32,f32) {
        return (PI / 3.0,PI / 4.0);
    }

    fn render(&self, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        let height = engine.viewport.height();
        let width = engine.viewport.width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];

        let mut settings_b:Vec<u8> = vec![];
        let gp = self.global_position();
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.x as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.y as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gp.z as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as f32)));

        let gd = self.global_direction().vec4;
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.w as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.x as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.y as f32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(gd.z as f32)));

        settings_b.extend_from_slice(bytemuck::bytes_of(&(width as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(height as u32)));

        settings_b.extend_from_slice(bytemuck::bytes_of(&self.angle_of_view().0));
        settings_b.extend_from_slice(bytemuck::bytes_of(&self.angle_of_view().1));

        settings_b.extend_from_slice(bytemuck::bytes_of(&(3 as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(self.sampling_size as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as u32)));
        settings_b.extend_from_slice(bytemuck::bytes_of(&(0 as u32)));


        let global_polygons = &engine.genesis.borrow_mut().global_polygons_recursive();

        let lights = &engine.lights;
        let mut lights_b:Vec<u8> = vec![0;32 * lights.len()];
        for i in 0..lights.len(){
            write_light_into_u8s(&mut lights_b[(i * 32)..((i + 1) * 32)],&RefCell::borrow(&lights[i]));
        }

        let particles = &engine.particles;
        let mut particles_b = vec![0;if particles.len() != 0 {particles.len() * 64} else { 64 }];
        for i in 0..particles.len(){
            write_particle_into_u8s(&mut particles_b[(i * 64)..((i + 1) * 64)],&particles[i]);
        }

        // insert a phantom particle in order to avoid shader error
        if particles.len() == 0{
            write_particle_into_u8s(&mut particles_b[0..64],&CParticle {
                position: Default::default(),
                velocity: Default::default(),
                color: Default::default(),
                c: ' ',
                threshold: 0.0,
                mode: SPHERE,
            });
        }

        let mut char_mapper_b:Vec<u8> = vec![0;4 * 256];
        for i in 0..256{
            char_mapper_b[(i * 4)..(i * 4 + 4)].copy_from_slice(bytemuck::bytes_of(&(self.cm.mem[i] as u32)));
        }

        if let Some((device,queue)) = &engine.wgpu_daq{
            let settings_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: settings_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let mut polygons_u8s = vec![0;(80 * global_polygons.len())];
            for i in 0..global_polygons.len(){
                write_polygon_into_u8s(&mut polygons_u8s[(i * 80)..((i + 1) * 80)],&global_polygons[i]);
            }

            let polygons_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: polygons_u8s.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let lights_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: lights_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let particles_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: particles_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let cp_counter_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: if particles.len() != 0 {particles.len() * 4} else { 4 } as u64,
                usage: wgpu::BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let char_mapper_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: char_mapper_b.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let bvh_tree_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: (global_polygons.len() * 4 * 32) as u64,
                usage: wgpu::BufferUsages::STORAGE| BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let bvh_flag_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: (global_polygons.len() * 4 * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let dots_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height * if self.sampling_size == 1 {1} else {8}) * 12) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let cp_results_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 12) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }
            );

            let chars_buffer_storage = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 4) as u64,
                    usage: (BufferUsages::STORAGE | BufferUsages::COPY_SRC),
                    mapped_at_creation: false,
                }
            );

            let chars_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((engine.viewport.width() * engine.viewport.height()) * 4) as u64,
                    usage: (wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                }
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some("ASCIA") });

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &self.init_bvh_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 1,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 4,
                            resource: bvh_tree_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 5,
                            resource: bvh_flag_buffer.as_entire_binding(),
                        },
                    ],
                });

                let mut dispatch_num = 1;
                while (dispatch_num * 64) < global_polygons.len(){
                    dispatch_num <<= 1;
                }
                let mut now_width = dispatch_num << 6;

                while dispatch_num >= 1{
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("BVH") });
                    cpass.set_pipeline(&self.init_bvh_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.insert_debug_marker("awww");
                    cpass.dispatch_workgroups(dispatch_num as u32,1,1);
                    dispatch_num >>= 6;
                    now_width >>= 6;
                    if dispatch_num == 0 && now_width >= 1{
                        dispatch_num = 1;
                    }
                }
            }

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &self.render_cp_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 6,
                            resource: particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 7,
                            resource: cp_counter_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &self.render_cp_pipeline.get_bind_group_layout(1) ,
                    entries: &[
                        BindGroupEntry{
                            binding: 2,
                            resource: cp_results_buffer_storage.as_entire_binding(),
                        },
                    ],
                });

                for i in 0..self.particle_rerender_num{
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER_CP") });
                    cpass.set_pipeline(&self.render_cp_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.insert_debug_marker("awww");
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            let render_pipeline = if self.sampling_size == 3 { &self.render_pipeline_3x } else {&self.render_pipeline_1x};
            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &render_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: lights_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: char_mapper_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 4,
                            resource: bvh_tree_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 6,
                            resource: particles_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &render_pipeline.get_bind_group_layout(1) ,
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: dots_buffer_storage.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: chars_buffer_storage.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: cp_results_buffer_storage.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(render_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.insert_debug_marker("awww");
                    if self.sampling_size == 1{
                        cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                    }
                    else{
                        cpass.dispatch_workgroups((width / 4 + 1) as u32,(height / 2 + 1) as u32,1);
                    }
                }
            }

            #[cfg(debug_assertions)]
            let bvh_tree_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: bvh_tree_buffer.size() as u64,
                    usage: (wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                }
            );

            #[cfg(debug_assertions)]
            let bvh_flag_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: bvh_flag_buffer.size() as u64,
                    usage: (wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
                    mapped_at_creation: false,
                }
            );


            encoder.copy_buffer_to_buffer(&chars_buffer_storage,0,&chars_buffer_staging,0,chars_buffer_storage.size());

            #[cfg(debug_assertions)]
            encoder.copy_buffer_to_buffer(&bvh_tree_buffer,0,&bvh_tree_buffer_staging,0,bvh_tree_buffer.size());

            #[cfg(debug_assertions)]
            encoder.copy_buffer_to_buffer(&bvh_flag_buffer,0,&bvh_flag_buffer_staging,0,bvh_flag_buffer.size());

            queue.submit(Some(encoder.finish()));

            #[cfg(debug_assertions)]
            let bvh_tree_dump_slice = bvh_tree_buffer_staging.slice(..);

            #[cfg(debug_assertions)]
            let (sender_bvh,receiver_bvh) = futures_intrusive::channel::shared::oneshot_channel();

            #[cfg(debug_assertions)]
            bvh_tree_dump_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender_bvh.send(v).unwrap()
            });

            #[cfg(debug_assertions)]
            let bvh_flag_dump_slice = bvh_flag_buffer_staging.slice(..);

            #[cfg(debug_assertions)]
            let (sender_bvh_flag,receiver_bvh_flag) = futures_intrusive::channel::shared::oneshot_channel();

            #[cfg(debug_assertions)]
            bvh_flag_dump_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender_bvh_flag.send(v).unwrap()
            });

            let chars_results_slice = chars_buffer_staging.slice(..);
            let (sender,receiver) = futures_intrusive::channel::shared::oneshot_channel();
            chars_results_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender.send(v).unwrap()
            });

            device.poll(wgpu::Maintain::Wait);

            #[cfg(debug_assertions)]
            if let Some(Ok(())) = pollster::block_on(receiver_bvh.receive()){
                let s = bvh_tree_dump_slice.get_mapped_range();
                let results:Vec<f32> = bytemuck::cast_slice(&s).to_vec();
                for i in 1..min(4096,global_polygons.len()){
                    for j in 0..3{
                        assert!(
                            (results[(i * 2) * 8 + j] == 0.0 && results[i * 8 + j] == results[(i * 2 + 1) * 8 + j])
                                || (results[(i * 2 + 1) * 8 + j] == 0.0 && results[i * 8 + j] == results[(i * 2) * 8 + j])
                                || (results[i * 8 + j] == f32::min(results[(i * 2) * 8 + j], results[(i * 2 + 1) * 8 + j])));
                    }
                    for j in 4..7{
                        assert!(
                            (results[(i * 2) * 8 + j] == 0.0 && results[i * 8 + j] == results[(i * 2 + 1) * 8 + j])
                                || (results[(i * 2 + 1) * 8 + j] == 0.0 && results[i * 8 + j] == results[(i * 2) * 8 + j])
                                || (results[i * 8 + j] == f32::max(results[(i * 2) * 8 + j], results[(i * 2 + 1) * 8 + j])));
                    }
                }
            }

            if let Some(Ok(())) = pollster::block_on(receiver.receive()){
                let s = chars_results_slice.get_mapped_range();
                let results:Vec<u32> = bytemuck::cast_slice(&s).to_vec();
                for x in 0..width{
                    for y in 0..height{
                        let result = results[y * width + x];
                        out[y][x] = RenderChar{
                            c: ((result & 0xff) as u8) as char,
                            color: ColorRGBu8 {
                                r: ((result >> 24) & 0xff) as u8,
                                g: ((result >> 16) & 0xff) as u8,
                                b: ((result >> 8) & 0xff) as u8,
                            },
                        }
                    }
                }
            }
        }
        return out;
    }
}

#[inline]
fn write_vec3_into_u8s(dst: &mut [u8], src: &Vec3, padding_len:usize){
    dst[0..4].copy_from_slice(bytemuck::bytes_of(&(src.x as f32)));
    dst[4..8].copy_from_slice(bytemuck::bytes_of(&(src.y as f32)));
    dst[8..12].copy_from_slice(bytemuck::bytes_of(&(src.z as f32)));
    for i in 0..padding_len{
        dst[(12 + i * 4)..(16 + i * 4)].copy_from_slice(bytemuck::bytes_of(&(0.0f32)));
    }
}

#[inline]
fn write_mat33_into_u8s(dst: &mut [u8], src: &Matrix33){
    write_vec3_into_u8s(&mut dst[0..16],&src.v1,1);
    write_vec3_into_u8s(&mut dst[16..32],&src.v2,1);
    write_vec3_into_u8s(&mut dst[32..48],&src.v3,1);
}

fn write_material_into_u8s(dst: &mut [u8], src: &Material){
    dst[0..4].copy_from_slice(bytemuck::bytes_of(&(src.color.r as f32)));
    dst[4..8].copy_from_slice(bytemuck::bytes_of(&(src.color.g as f32)));
    dst[8..12].copy_from_slice(bytemuck::bytes_of(&(src.color.b as f32)));
    dst[12..16].copy_from_slice(bytemuck::bytes_of(&(src.mode as u32)));

    dst[16..20].copy_from_slice(bytemuck::bytes_of(&(src.priority as f32)));
    dst[20..24].copy_from_slice(bytemuck::bytes_of(&(0.0f32)));
    dst[24..28].copy_from_slice(bytemuck::bytes_of(&(0.0f32)));
    dst[28..32].copy_from_slice(bytemuck::bytes_of(&(0.0f32)));
}

#[inline]
fn write_polygon_into_u8s(dst: &mut [u8], src: &Polygon){
    write_mat33_into_u8s(&mut dst[0..48],&src.poses);
    write_material_into_u8s(&mut dst[48..80],&src.material);
}

fn write_light_into_u8s(dst: &mut [u8], src: &Light){
    write_vec3_into_u8s(&mut dst[0..16],&src.global_position(),1);
    dst[16..20].copy_from_slice(bytemuck::bytes_of(&(src.color.r as f32)));
    dst[20..24].copy_from_slice(bytemuck::bytes_of(&(src.color.g as f32)));
    dst[24..28].copy_from_slice(bytemuck::bytes_of(&(src.color.b as f32)));
    dst[28..32].copy_from_slice(bytemuck::bytes_of(&(src.power as f32)));
}

fn write_particle_into_u8s(dst: &mut [u8], src: &CParticle){
    write_vec3_into_u8s(&mut dst[0..16],&src.position,1);
    write_vec3_into_u8s(&mut dst[16..32],&src.velocity,1);

    dst[32..36].copy_from_slice(bytemuck::bytes_of(&(src.color.r as f32)));
    dst[36..40].copy_from_slice(bytemuck::bytes_of(&(src.color.g as f32)));
    dst[40..44].copy_from_slice(bytemuck::bytes_of(&(src.color.b as f32)));
    dst[44..48].copy_from_slice(bytemuck::bytes_of(&(src.c as u32)));

    dst[48..52].copy_from_slice(bytemuck::bytes_of(&(src.threshold as f32)));
    dst[52..56].copy_from_slice(bytemuck::bytes_of(&(src.mode as u32)));
    dst[56..60].copy_from_slice(bytemuck::bytes_of(&(0 as f32)));
    dst[60..64].copy_from_slice(bytemuck::bytes_of(&(0 as f32)));
}