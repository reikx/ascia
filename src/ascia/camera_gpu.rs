use std::borrow::Cow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BufferDescriptor, ShaderModule, ShaderModuleDescriptor};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use crate::ascia::charmapper::CHARMAP3X3;
use crate::ascia::color::{ColorRGBf32, ColorRGBu8};
use crate::ascia::core::{AsciaEngine, Camera, CParticle, FlatMaterial, Global, LambertMaterial, LambertWithShadowMaterial, ObjectNode, PresetObjectNodeAttribute, Polygon, PresetLight, PresetMaterial, RenderChar, ObjectNodeAttribute, PresetCamera};
use crate::ascia::core::CParticleMode::SPHERE;
use crate::ascia::lights::PointLight;
use crate::ascia::math::{Matrix33, Quaternion, Vec3, Vec4};

trait GPUMemoryConvertStatic<const N:usize>{
    fn convert(&self) -> [u8; N];

    #[inline]
    fn convert_len() -> usize{
        return N;
    }
}

impl GPUMemoryConvertStatic<4> for u32{

    #[inline]
    fn convert(&self) -> [u8; 4] {
        return unsafe {
            let buf:[u8; 4] = mem::transmute(*self);
            buf
        }
    }
}

impl GPUMemoryConvertStatic<4> for f32{

    #[inline]
    fn convert(&self) -> [u8; 4] {
        return unsafe {
            let buf:[u8; 4] = mem::transmute(*self);
            buf
        }
    }
}

impl GPUMemoryConvertStatic<4> for char{

    #[inline]
    fn convert(&self) -> [u8; 4] {
        return unsafe {
            let buf:[u8; 4] = mem::transmute(*self);
            buf
        }
    }
}

impl GPUMemoryConvertStatic<12> for Vec3{

    #[inline]
    fn convert(&self) -> [u8; 12] {
        return unsafe {
            let buf:[u8; 12] = mem::transmute(*self);
            buf
        }
    }
}
impl GPUMemoryConvertStatic<16> for Vec4{

    #[inline]
    fn convert(&self) -> [u8; 16] {
        return unsafe {
            let buf:[u8; 16] = mem::transmute(*self);
            buf
        }
    }
}

impl GPUMemoryConvertStatic<16> for Quaternion{

    #[inline]
    fn convert(&self) -> [u8; 16] {
        return self.vec4.convert();
    }
}

impl GPUMemoryConvertStatic<48> for Matrix33{

    #[inline]
    fn convert(&self) -> [u8; 48] {
        let mut buf:[u8; 48] = [0; 48];
        buf[0..12].copy_from_slice(&self.v1.convert());
        buf[16..28].copy_from_slice(&self.v2.convert());
        buf[32..44].copy_from_slice(&self.v3.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<12> for ColorRGBf32{

    #[inline]
    fn convert(&self) -> [u8; 12] {
        return unsafe {
            let buf:[u8; 12] = mem::transmute(*self);
            buf
        }
    }
}

impl GPUMemoryConvertStatic<20> for FlatMaterial{
    fn convert(&self) -> [u8; 20] {
        let mut buf:[u8; 20] = [0; 20];
        buf[0..12].copy_from_slice(&self.color.convert());
        buf[12..16].copy_from_slice(&0u32.convert());
        buf[16..20].copy_from_slice(&self.priority.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<20> for LambertMaterial{
    fn convert(&self) -> [u8; 20] {
        let mut buf:[u8; 20] = [0; 20];
        buf[0..12].copy_from_slice(&self.color.convert());
        buf[12..16].copy_from_slice(&1u32.convert());
        buf[16..20].copy_from_slice(&self.priority.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<20> for LambertWithShadowMaterial{
    fn convert(&self) -> [u8; 20] {
        let mut buf:[u8; 20] = [0; 20];
        buf[0..12].copy_from_slice(&self.color.convert());
        buf[12..16].copy_from_slice(&2u32.convert());
        buf[16..20].copy_from_slice(&self.priority.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<20> for PresetMaterial{

    #[inline]
    fn convert(&self) -> [u8; 20] {
        let mut buf:[u8; 20] = [0; 20];
        match self {
            PresetMaterial::Flat(m) => {
                buf[0..20].copy_from_slice(&m.convert());
            }
            PresetMaterial::Lambert(m) => {
                buf[0..20].copy_from_slice(&m.convert());
            }
            PresetMaterial::LambertWithShadow(m) => {
                buf[0..20].copy_from_slice(&m.convert());
            }
        }
        return buf;
    }
}

impl GPUMemoryConvertStatic<80> for Polygon<Global>{
    #[inline]
    fn convert(&self) -> [u8; 80] {
        let mut buf:[u8; 80] = [0; 80];
        buf[0..48].copy_from_slice(&self.poses.convert());
        buf[48..68].copy_from_slice(&self.material.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<64> for CParticle<Global>{
    #[inline]
    fn convert(&self) -> [u8; 64] {
        let mut buf:[u8; 64] = [0; 64];
        buf[0..12].copy_from_slice(&self.position.convert());
        buf[16..28].copy_from_slice(&self.velocity.convert());
        buf[32..44].copy_from_slice(&self.color.convert());
        buf[44..48].copy_from_slice(&self.c.convert());
        buf[48..52].copy_from_slice(&self.threshold.convert());
        buf[52..56].copy_from_slice(&(self.mode as u32).convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<16> for PointLight{
    #[inline]
    fn convert(&self) -> [u8; 16] {
        let mut buf:[u8; 16] = [0; 16];
        buf[0..12].copy_from_slice(&self.color.convert());
        buf[12..16].copy_from_slice(&self.power.convert());
        return buf;
    }
}

impl GPUMemoryConvertStatic<32> for (Vec3, PointLight){
    fn convert(&self) -> [u8; 32] {
        let mut buf:[u8; 32] = [0; 32];
        buf[0..12].copy_from_slice(&self.0.convert());
        buf[16..32].copy_from_slice(&self.1.convert());
        return buf;
    }
}

impl<const N:usize,G:GPUMemoryConvertStatic<N>> GPUMemoryConvertStatic<N> for &G{
    #[inline]
    fn convert(&self) -> [u8; N] {
        return self.convert();
    }
}


trait GPUMemoryConvertDynamic<const N:usize>{
    fn convert(&self) -> Vec<u8>;
}

impl<const N: usize, G:GPUMemoryConvertStatic<N>> GPUMemoryConvertDynamic<N> for Vec<G>{
    fn convert(&self) -> Vec<u8> {

        let mut buf = Vec::with_capacity(16 * self.len());
        for p in self{
            buf.extend(&p.convert());
        }
        return buf;
    }
}

impl<const N: usize, G:GPUMemoryConvertStatic<N>> GPUMemoryConvertDynamic<N> for &[G]{
    fn convert(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 * self.len());
        for p in *self{
            buf.extend(&p.convert());
        }
        return buf;
    }
}


struct RaytracingSetting{
    camera_position:Vec3,
    camera_direction:Quaternion,
    screen_width:u32,
    screen_height:u32,
    angle_of_view:(f32,f32),
    max_reflection:u32,
    sampling_size:u32,
    sampling_threshold:u32,
    render_range_radius:f32
}


impl GPUMemoryConvertStatic<64> for RaytracingSetting{

    #[inline]
    fn convert(&self) -> [u8; 64] {
        let mut buf:[u8; 64] = [0; 64];
        buf[0..12].copy_from_slice(&self.camera_position.convert());
        buf[16..32].copy_from_slice(&self.camera_direction.convert());
        buf[32..36].copy_from_slice(&self.screen_width.convert());
        buf[36..40].copy_from_slice(&self.screen_height.convert());
        buf[40..44].copy_from_slice(&self.angle_of_view.0.convert());
        buf[44..48].copy_from_slice(&self.angle_of_view.1.convert());
        buf[48..52].copy_from_slice(&self.max_reflection.convert());
        buf[52..56].copy_from_slice(&self.sampling_size.convert());
        buf[56..60].copy_from_slice(&self.sampling_threshold.convert());
        buf[60..64].copy_from_slice(&self.render_range_radius.convert());
        return buf;
    }
}

pub struct GPUWrapper<C:Camera>{
    pub cpu_camera: C,
    shader: ShaderModule,
    _ph: PhantomData<C>
}

impl GPUWrapper<SimpleCamera>{
    pub fn generate(value: SimpleCamera, engine: &AsciaEngine) -> Self {
        let daq = if let Some(d) = &engine.wgpu_daq{
            d
        } else {
            panic!();
        };
        let device = &daq.0;
        return GPUWrapper{
            cpu_camera: value,
            shader: device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simple_raytracing_renderer.wgsl")))
            }),
            _ph: Default::default()
        };
    }
}


impl ObjectNodeAttribute for GPUWrapper<SimpleCamera> {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Camera(PresetCamera::SimpleGPU(self))));
    }
}

impl Camera for GPUWrapper<SimpleCamera>{
    fn render(&self, node: &ObjectNode<Global>, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        let daq = if let Some(d) = &engine.wgpu_daq{
            d
        } else {
            panic!();
        };
        let device = &daq.0;
        let calc_intersections_polygons_1x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_polygons_1x",
        });
        let calc_intersections_polygons_3x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_polygons_3x",
        });
        let calc_intersections_c_particles_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_c_particles",
        });
        let calc_chars_1x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_chars_1x",
        });
        let calc_chars_3x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_chars_3x",
        });

        let height = engine.viewport.borrow().height();
        let width = engine.viewport.borrow().width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];

        let mut polygons = vec![];
        let mut c_particles = vec![];
        let mut pointlights = vec![];
        for iter in engine.genesis_global.iter(){
            polygons.extend(iter.polygons.clone());
            c_particles.extend(iter.c_particles.clone());
            let attr_rr = RefCell::borrow(&iter.attribute);
            if let PresetObjectNodeAttribute::Light(light) = &*attr_rr{
                if let PresetLight::Point(p) = light{
                    pointlights.push((iter.position, PointLight{
                        color: p.color,
                        power: p.power,
                    }));
                }
            }
        }

        // insert a phantom polygon in order to avoid shader error
        if polygons.is_empty(){
            polygons.push(Polygon::new(&Default::default(),&Default::default(),&Default::default()));
        }

        // insert a phantom particle in order to avoid shader error
        if c_particles.is_empty(){
            c_particles.push(CParticle {
                position: Default::default(),
                velocity: Default::default(),
                color: Default::default(),
                c: ' ',
                threshold: 0.0,
                mode: SPHERE,
                _ph: Default::default(),
            });
        }

        // insert a phantom particle in order to avoid shader error
        if pointlights.is_empty(){
            pointlights.push((Vec3::default(), PointLight {
                color: Default::default(),
                power: 0.0,
            }));
        }


        if let Some((device,queue)) = &engine.wgpu_daq{
            let settings_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &RaytracingSetting{
                    camera_position: node.position,
                    camera_direction: node.direction,
                    screen_width: width as u32,
                    screen_height: height as u32,
                    angle_of_view: self.cpu_camera.angle_of_view,
                    max_reflection: 0,
                    sampling_size: self.cpu_camera.sampling_size,
                    sampling_threshold: 0,
                    render_range_radius: 10000.0,
                }.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let char_mapper_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &(&CHARMAP3X3[..]).convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let rendered_chars_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height) * 4) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let rendered_chars_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height) * 4) as u64,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }
            );

            let polygons_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &polygons.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let intersections_polygons_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * self.cpu_camera.sampling_size as usize) * (height * self.cpu_camera.sampling_size as usize) * 96) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );

            let c_particles_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &c_particles.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let intersections_c_particles_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * self.cpu_camera.sampling_size as usize) * (height * self.cpu_camera.sampling_size as usize) * 64) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let c_particle_counters_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: (c_particles.len() * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let pointlights_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &pointlights.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some("ASCIA") });
            
            let calc_intersections_polygons_pipeline = match self.cpu_camera.sampling_size {
                1 => {&calc_intersections_polygons_1x_pipeline }
                3 => {&calc_intersections_polygons_3x_pipeline }
                _ => {panic!()}
            };

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_polygons_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(calc_intersections_polygons_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    match self.cpu_camera.sampling_size {
                        1 => {
                            cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                        }
                        3 => {
                            cpass.dispatch_workgroups((width / 4 + 1) as u32,(height / 2 + 1) as u32,1);
                        }
                        _ => {
                            panic!();
                        }
                    }
                }
            }

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(2),
                    entries: &[],
                });
                let bind_group_3 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(3),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 4,
                            resource: c_particle_counters_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(&calc_intersections_c_particles_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    cpass.set_bind_group(3,&bind_group_3,&[]);
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            let calc_chars_pipeline = match self.cpu_camera.sampling_size {
                1 => {&calc_chars_1x_pipeline }
                3 => {&calc_chars_3x_pipeline }
                _ => {panic!()}
            };

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: char_mapper_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: rendered_chars_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_polygons_buffer.as_entire_binding(),
                        }
                    ],
                });
                let bind_group_3 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(3),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_c_particles_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_4 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(4),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: pointlights_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(&calc_chars_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    cpass.set_bind_group(3,&bind_group_3,&[]);
                    cpass.set_bind_group(4,&bind_group_4,&[]);
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            encoder.copy_buffer_to_buffer(&rendered_chars_buffer,0,&rendered_chars_buffer_staging,0, rendered_chars_buffer.size());

            queue.submit(Some(encoder.finish()));

            let chars_results_slice = rendered_chars_buffer_staging.slice(..);
            let (sender,receiver) = futures_intrusive::channel::shared::oneshot_channel();
            chars_results_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender.send(v).unwrap()
            });

            device.poll(wgpu::Maintain::Wait);

            if let Some(Ok(())) = pollster::block_on(receiver.receive()){
                let s = &chars_results_slice.get_mapped_range();
                for x in 0..width{
                    for y in 0..height{
                        let offset = (y * width + x) * 4;
                        out[y][x] = RenderChar{
                            c: s[offset + 0] as char,
                            color: ColorRGBu8 {
                                r: s[offset + 3],
                                g: s[offset + 2],
                                b: s[offset + 1],
                            },
                        };
                    }
                }
            }
        }
        return out;
    }
}

impl GPUWrapper<SimpleBVHCamera>{
    pub fn generate(value: SimpleBVHCamera, engine: &AsciaEngine) -> Self {
        let daq = if let Some(d) = &engine.wgpu_daq{
            d
        } else {
            panic!();
        };
        let device = &daq.0;
        return GPUWrapper{
            cpu_camera: value,
            shader: device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simple_raytracing_renderer_naive_bvh.wgsl")))
            }),
            _ph: Default::default()
        };
    }
}

impl ObjectNodeAttribute for GPUWrapper<SimpleBVHCamera> {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Camera(PresetCamera::SimpleBVHGPU(self))));
    }
}

impl Camera for GPUWrapper<SimpleBVHCamera>{
    fn render(&self, node: &ObjectNode<Global>, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        let daq = if let Some(d) = &engine.wgpu_daq{
            d
        } else {
            panic!();
        };
        let device = &daq.0;
        let build_bvh_polygons_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "build_bvh_polygons",
        });
        let build_bvh_c_particles_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "build_bvh_c_particles",
        });
        let calc_intersections_polygons_1x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_polygons_1x",
        });
        let calc_intersections_polygons_3x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_polygons_3x",
        });
        let calc_intersections_c_particles_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_intersections_c_particles",
        });
        let calc_chars_1x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_chars_1x",
        });
        let calc_chars_3x_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &self.shader,
            entry_point: "calc_chars_3x",
        });

        let height = engine.viewport.borrow().height();
        let width = engine.viewport.borrow().width();
        let mut out:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];

        let mut polygons = vec![];
        let mut c_particles = vec![];
        let mut pointlights = vec![];
        for iter in engine.genesis_global.iter(){
            polygons.extend(iter.polygons.clone());
            c_particles.extend(iter.c_particles.clone());
            let attr_rr = RefCell::borrow(&iter.attribute);
            if let PresetObjectNodeAttribute::Light(light) = &*attr_rr{
                if let PresetLight::Point(p) = light{
                    pointlights.push((iter.position, PointLight{
                        color: p.color,
                        power: p.power,
                    }));
                }
            }
        }

        // insert a phantom polygon in order to avoid shader error
        if polygons.is_empty(){
            polygons.push(Polygon::new(&Default::default(),&Default::default(),&Default::default()));
        }

        // insert a phantom particle in order to avoid shader error
        if c_particles.is_empty(){
            c_particles.push(CParticle {
                position: Default::default(),
                velocity: Default::default(),
                color: Default::default(),
                c: ' ',
                threshold: 0.0,
                mode: SPHERE,
                _ph: Default::default(),
            });
        }

        // insert a phantom particle in order to avoid shader error
        if pointlights.is_empty(){
            pointlights.push((Vec3::default(), PointLight {
                color: Default::default(),
                power: 0.0,
            }));
        }


        if let Some((device,queue)) = &engine.wgpu_daq{
            let settings_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &RaytracingSetting{
                    camera_position: node.position,
                    camera_direction: node.direction,
                    screen_width: width as u32,
                    screen_height: height as u32,
                    angle_of_view: self.cpu_camera.angle_of_view,
                    max_reflection: 0,
                    sampling_size: self.cpu_camera.sampling_size,
                    sampling_threshold: 0,
                    render_range_radius: 10000.0,
                }.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let char_mapper_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &(&CHARMAP3X3[..]).convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let rendered_chars_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height) * 4) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let rendered_chars_buffer_staging = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * height) * 4) as u64,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }
            );

            let polygons_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &polygons.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let bvh_tree_polygons_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: (polygons.len() * 4 * 32) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let bvh_flag_polygons_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: (polygons.len() * 4 * 4) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let intersections_polygons_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * self.cpu_camera.sampling_size as usize) * (height * self.cpu_camera.sampling_size as usize) * 96) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );

            let c_particles_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &c_particles.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
            let bvh_tree_c_particles_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: (c_particles.len() * 4 * 32) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let bvh_flag_c_particles_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: (c_particles.len() * 4 * 4) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let intersections_c_particles_buffer = device.create_buffer(
                &BufferDescriptor{
                    label: None,
                    size: ((width * self.cpu_camera.sampling_size as usize) * (height * self.cpu_camera.sampling_size as usize) * 64) as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }
            );
            let c_particle_counters_buffer = device.create_buffer(&BufferDescriptor{
                label: None,
                size: (c_particles.len() * 4) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });


            let pointlights_buffer = device.create_buffer_init(&BufferInitDescriptor{
                label: None,
                contents: &pointlights.convert(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some("ASCIA") });

            // build_bvh_polygons
            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_polygons_pipeline.get_bind_group_layout(0),
                    entries: &[],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_polygons_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_polygons_pipeline.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: bvh_tree_polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: bvh_flag_polygons_buffer.as_entire_binding(),
                        },
                    ],
                });

                let mut dispatch_num = 1;
                while (dispatch_num * 64) < polygons.len(){
                    dispatch_num <<= 1;
                }
                let mut now_width = dispatch_num << 6;

                while dispatch_num >= 1{
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("BVH") });
                    cpass.set_pipeline(&build_bvh_polygons_pipeline);
                    cpass.set_bind_group(0, &bind_group_0,&[]);
                    cpass.set_bind_group(1, &bind_group_1,&[]);
                    cpass.set_bind_group(2, &bind_group_2,&[]);
                    cpass.dispatch_workgroups(dispatch_num as u32,1,1);
                    dispatch_num >>= 6;
                    now_width >>= 6;
                    if dispatch_num == 0 && now_width >= 1{
                        dispatch_num = 1;
                    }
                }
            }

            // build_bvh_c_particles
            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_c_particles_pipeline.get_bind_group_layout(0),
                    entries: &[BindGroupEntry{
                        binding: 0,
                        resource: settings_buffer.as_entire_binding(),
                    }],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_c_particles_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_c_particles_pipeline.get_bind_group_layout(2),
                    entries: &[],
                });
                let bind_group_3 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &build_bvh_c_particles_pipeline.get_bind_group_layout(3),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: bvh_tree_c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 2,
                            resource: bvh_flag_c_particles_buffer.as_entire_binding(),
                        }
                    ],
                });

                let mut dispatch_num = 1;
                while (dispatch_num * 64) < c_particles.len(){
                    dispatch_num <<= 1;
                }
                let mut now_width = dispatch_num << 6;

                while dispatch_num >= 1{
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("BVH") });
                    cpass.set_pipeline(&build_bvh_c_particles_pipeline);
                    cpass.set_bind_group(0, &bind_group_0,&[]);
                    cpass.set_bind_group(1, &bind_group_1,&[]);
                    cpass.set_bind_group(2, &bind_group_2,&[]);
                    cpass.set_bind_group(3, &bind_group_3,&[]);
                    cpass.dispatch_workgroups(dispatch_num as u32,1,1);
                    dispatch_num >>= 6;
                    now_width >>= 6;
                    if dispatch_num == 0 && now_width >= 1{
                        dispatch_num = 1;
                    }
                }
            }

            let calc_intersections_polygons_pipeline = match self.cpu_camera.sampling_size {
                1 => {&calc_intersections_polygons_1x_pipeline }
                3 => {&calc_intersections_polygons_3x_pipeline }
                _ => {panic!()}
            };

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_polygons_pipeline.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: bvh_tree_polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_polygons_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(calc_intersections_polygons_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    match self.cpu_camera.sampling_size {
                        1 => {
                            cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                        }
                        3 => {
                            cpass.dispatch_workgroups((width / 4 + 1) as u32,(height / 2 + 1) as u32,1);
                        }
                        _ => {
                            panic!();
                        }
                    }
                }
            }

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(1),
                    entries: &[],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(2),
                    entries: &[],
                });
                let bind_group_3 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_intersections_c_particles_pipeline.get_bind_group_layout(3),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: bvh_tree_c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 4,
                            resource: c_particle_counters_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(&calc_intersections_c_particles_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    cpass.set_bind_group(3,&bind_group_3,&[]);
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            let calc_chars_pipeline = match self.cpu_camera.sampling_size {
                1 => {&calc_chars_1x_pipeline }
                3 => {&calc_chars_3x_pipeline }
                _ => {panic!()}
            };

            {
                let bind_group_0 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(0),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: settings_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: char_mapper_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_1 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(1),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: rendered_chars_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_2 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(2),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 1,
                            resource: bvh_tree_polygons_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_polygons_buffer.as_entire_binding(),
                        }
                    ],
                });
                let bind_group_3 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(3),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: c_particles_buffer.as_entire_binding(),
                        },
                        BindGroupEntry{
                            binding: 3,
                            resource: intersections_c_particles_buffer.as_entire_binding(),
                        },
                    ],
                });
                let bind_group_4 = device.create_bind_group(&BindGroupDescriptor{
                    label: None,
                    layout: &calc_chars_pipeline.get_bind_group_layout(4),
                    entries: &[
                        BindGroupEntry{
                            binding: 0,
                            resource: pointlights_buffer.as_entire_binding(),
                        },
                    ],
                });

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("RENDER") });
                    cpass.set_pipeline(&calc_chars_pipeline);
                    cpass.set_bind_group(0,&bind_group_0,&[]);
                    cpass.set_bind_group(1,&bind_group_1,&[]);
                    cpass.set_bind_group(2,&bind_group_2,&[]);
                    cpass.set_bind_group(3,&bind_group_3,&[]);
                    cpass.set_bind_group(4,&bind_group_4,&[]);
                    cpass.dispatch_workgroups((width / 8 + 1) as u32,(height / 8 + 1) as u32,1);
                }
            }

            encoder.copy_buffer_to_buffer(&rendered_chars_buffer,0,&rendered_chars_buffer_staging,0, rendered_chars_buffer.size());

            queue.submit(Some(encoder.finish()));

            let chars_results_slice = rendered_chars_buffer_staging.slice(..);
            let (sender,receiver) = futures_intrusive::channel::shared::oneshot_channel();
            chars_results_slice.map_async(wgpu::MapMode::Read,move |v|{
                sender.send(v).unwrap()
            });

            device.poll(wgpu::Maintain::Wait);

            if let Some(Ok(())) = pollster::block_on(receiver.receive()){
                let s = &chars_results_slice.get_mapped_range();
                for x in 0..width{
                    for y in 0..height{
                        let offset = (y * width + x) * 4;
                        out[y][x] = RenderChar{
                            c: s[offset + 0] as char,
                            color: ColorRGBu8 {
                                r: s[offset + 3],
                                g: s[offset + 2],
                                b: s[offset + 1],
                            },
                        };
                    }
                }
            }
        }
        return out;
    }
}
