use std::cell::{Ref, RefCell, RefMut};
use std::char::{from_digit};
use std::io::{BufWriter, stdout, StdoutLock, Write};
use std::ops::Bound::{Excluded, Included};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};
use crate::ascia::core::EngineStatus::{Rendering, UpdatingEntities, UpdatingNodes, Waiting};
use crate::ascia::core::MaterialMode::{FLAT, LAMBERT, LAMBERT_WITH_SHADOW};
use crate::ascia::lights::Light;
use crate::ascia::math::{Matrix33, Quaternion, Vec2, Vec3};
use crate::ascia::raycaster::Raycaster;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Material{
    pub color: ColorRGBf32,
    pub mode: MaterialMode,
    pub priority: u32,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum MaterialMode{
    FLAT = 0u32,
    LAMBERT = 1u32,
    LAMBERT_WITH_SHADOW = 2u32,
    NONE = u32::MAX
}

#[derive(Copy, Clone)]
pub struct Ray{
    pub position:Vec3,
    pub direction:Vec3,
}

impl Material{
    #[inline]
    pub fn shade(&self, global_polygons:&Vec<Polygon>, _ray: &Ray, polygon_index:usize, _depth: f32, _intersection_position_on_polygon: &Vec2, intersection_position_global: &Vec3, camera_position: &Vec3, lights:&Vec<Rc<RefCell<Light>>>) -> ColorRGBf32 {
        if self.mode == FLAT{
            return self.color;
        }
        else if self.mode == LAMBERT{
            let mut result = ColorRGBf32{
                r:0.0,
                g:0.0,
                b:0.0
            };
            for l in lights{
                let light = RefCell::borrow(l);
                let normal = (global_polygons[polygon_index].poses.v2 - global_polygons[polygon_index].poses.v1) ^ (global_polygons[polygon_index].poses.v3 - global_polygons[polygon_index].poses.v1);
                let lhs = (*intersection_position_global - light.global_position()).normalize();
                let rhs = normal.normalize();
                let co = -(lhs * rhs);
                let color = light.ray(&intersection_position_global);
                if (co < 0.0 && normal * (*camera_position - *intersection_position_global) < 0.0) || (co > 0.0 && normal * (*camera_position - *intersection_position_global) > 0.0){
                    result += ColorRGBf32 {
                        r: ((self.color.r * color.r) * f32::abs(co)) as f32,
                        g: ((self.color.g * color.g) * f32::abs(co)) as f32,
                        b: ((self.color.b * color.b) * f32::abs(co)) as f32,
                    }
                }
            }
            return result;
        }
        else if self.mode == LAMBERT_WITH_SHADOW{
            let mut result = ColorRGBf32{
                r:0.0,
                g:0.0,
                b:0.0
            };
            for l in lights{
                let light = RefCell::borrow(l);
                let normal = (global_polygons[polygon_index].poses.v2 - global_polygons[polygon_index].poses.v1) ^ (global_polygons[polygon_index].poses.v3 - global_polygons[polygon_index].poses.v1);
                let lhs = (*intersection_position_global - light.global_position()).normalize();
                let rhs = normal.normalize();
                let co = -(lhs * rhs);
                let color = light.ray(&intersection_position_global);
                let mut is_prevented = false;
                for i in 0..global_polygons.len(){
                    if i == polygon_index{
                        continue;
                    }
                    if let Some(r) = Raycaster::project_polygon(intersection_position_global, &(light.global_position() - *intersection_position_global), i, global_polygons){
                        if r.depth >= 0.0{
                            is_prevented = true;
                            break;
                        }
                    }
                }
                if !is_prevented && ((co < 0.0 && normal * (*camera_position - *intersection_position_global) < 0.0) || (co > 0.0 && normal * (*camera_position - *intersection_position_global) > 0.0)){
                    result += ColorRGBf32 {
                        r: ((self.color.r * color.r) * f32::abs(co)) as f32,
                        g: ((self.color.g * color.g) * f32::abs(co)) as f32,
                        b: ((self.color.b * color.b) * f32::abs(co)) as f32,
                    }
                }
            }
            return result;
        }
        return ColorRGBf32::default();
    }
}

impl Default for Material{
    fn default() -> Self {
        return Material{
            color: ColorRGBf32::default(),
            mode: FLAT,
            priority: 0
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Polygon {
    pub poses:Matrix33,
    pub material:Material
}

impl Polygon {
    pub fn new(p1:&Vec3, p2:&Vec3, p3:&Vec3) -> Self{
        return Polygon {
            poses:Matrix33{
                v1:p1.clone(),
                v2:p2.clone(),
                v3:p3.clone()
            },
            material: Default::default()
        }
    }
}

pub struct ObjectNode{
    pub position:Vec3,
    global_position:Vec3,
    pub direction:Quaternion,
    global_direction:Quaternion,
    pub polygons:Vec<Polygon>,
    global_polygons:Vec<Polygon>,
    pub parent:Option<Weak<RefCell<ObjectNode>>>,
    pub children:Vec<Rc<RefCell<ObjectNode>>>
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CParticle {
    pub position:Vec3,
    pub velocity:Vec3,
    pub color:ColorRGBf32,
    pub c:char,
    pub threshold:f32,
    pub mode:CParticleMode
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum CParticleMode{
    SPHERE = 0u32,
    ARG = 1u32
}

impl ObjectNode{
    pub fn new() -> Self{
        return ObjectNode::from(vec![]);
    }

    pub fn from(polygons:Vec<Polygon>) -> Self {
        return ObjectNode{
            position:Vec3::default(),
            global_position:Vec3::default(),
            direction:Quaternion::default(),
            global_direction:Quaternion::default(),
            polygons:polygons,
            global_polygons:vec![],
            parent:None,
            children:vec![]
        };
    }

    pub fn generate() -> Rc<RefCell<Self>>{
        return Rc::new(RefCell::new(ObjectNode::new()));
    }

    pub fn connect(parent:&Rc<RefCell<Self>>,child:&Rc<RefCell<Self>>){
        (*child).borrow_mut().parent = Some(Rc::downgrade(parent));
        (*parent).borrow_mut().children.push(child.clone());
    }

    pub fn position(&self) -> Vec3{
        return self.position.clone();
    }

    pub fn direction(&self) -> Quaternion{
        return self.direction.clone();
    }

    pub fn global_position(&self) -> Vec3{ return self.global_position.clone()}

    pub fn global_direction(&self) -> Quaternion{ return self.global_direction.clone(); }

    pub fn global_polygons_recursive(&self) -> Vec<Polygon>{
        let mut vec:Vec<Polygon> = vec![];
        vec.extend_from_slice(self.global_polygons.as_slice());
        for c in &self.children{
            vec.extend_from_slice(RefCell::borrow(c).global_polygons_recursive().as_slice());
        }
        return vec;
    }

    pub fn update_global_properties_recursive(&self){
        for c in &self.children{
            let mut child = (*c).borrow_mut();

            child.global_position = (self.global_position) + (self.global_direction).rotate(&child.position);
            child.global_direction = (self.global_direction) * (child.direction);

            let len = child.polygons.len();

            child.global_polygons.resize(len,Polygon {
                poses: Matrix33{
                    v1: Default::default(),
                    v2: Default::default(),
                    v3: Default::default(),
                },
                material: Default::default()
            });

            for i in 0..len{
                (*child.global_polygons)[i].poses.v1 = child.global_direction.rotate(&child.polygons[i].poses.v1) + (child.global_position);
                (*child.global_polygons)[i].poses.v2 = child.global_direction.rotate(&child.polygons[i].poses.v2) + (child.global_position);
                (*child.global_polygons)[i].poses.v3 = child.global_direction.rotate(&child.polygons[i].poses.v3) + (child.global_position);
                (*child.global_polygons)[i].material = child.polygons[i].material;
            }

            child.update_global_properties_recursive();
        }
    }
}

pub trait Entity{

    /// returns representative reference to ObjectNode
    fn repr<'a>(& 'a self) -> &'a Rc<RefCell<ObjectNode>>;

    fn repr_ref<'a>(&'a self) -> Ref<'a,ObjectNode>{
        return self.repr().borrow();
    }
    fn repr_refmut<'a>(&'a mut self) -> RefMut<'a,ObjectNode>{
        return self.repr().borrow_mut();
    }

    fn position(&self) -> Vec3{
        return self.repr_ref().position;
    }

    fn global_position(&self) -> Vec3{
        return self.repr_ref().global_position;
    }

    fn direction(&self) -> Quaternion{
        return self.repr_ref().direction;
    }

    fn global_direction(&self) -> Quaternion{
        return self.repr_ref().global_direction;
    }

    fn update(&mut self,engine_time:&Duration);
}

pub trait CameraEntity:Entity{
    fn angle_of_view(&self) -> (f32,f32);
    fn render(&self, engine:&AsciaEngine) -> Vec<Vec<RenderChar>>;
}

#[derive(Copy, Clone)]
pub struct RenderChar{
    pub(crate) c:char,
    pub(crate) color: ColorRGBu8
}

impl Default for RenderChar{
    fn default() -> Self {
        return RenderChar{
            c: ' ',
            color: ColorRGBu8::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Color8bit {
    data: u8,
}

impl From<ColorRGBu8> for Color8bit{
    fn from(value: ColorRGBu8) -> Self{
        if value.r == value.g && value.g == value.b{
            return Color8bit{
                data:232 + value.r / 11
            }
        }
        return Color8bit{
            data:16 + (value.r as f64 / (256.0 / 6.0)) as u8 * 36 + (value.g as f64 / (256.0 / 6.0)) as u8 * 6 + (value.b as f64 / (256.0 / 6.0)) as u8
        }
    }
}

#[derive(Copy, Clone)]
pub struct ColorRGBu8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for ColorRGBu8 {
    fn default() -> Self {
        return ColorRGBu8 {
            r:0,
            g:0,
            b:0
        }
    }
}

impl std::ops::AddAssign<ColorRGBu8> for ColorRGBu8 {
    fn add_assign(&mut self, rhs: ColorRGBu8) {
        self.r = self.r.saturating_add(rhs.r);
        self.g = self.g.saturating_add(rhs.g);
        self.b = self.b.saturating_add(rhs.b);
    }
}

impl std::ops::SubAssign<ColorRGBu8> for ColorRGBu8 {
    fn sub_assign(&mut self, rhs: ColorRGBu8) {
        self.r = self.r.saturating_sub(rhs.r);
        self.g = self.g.saturating_sub(rhs.g);
        self.b = self.b.saturating_sub(rhs.b);
    }
}

impl std::ops::Add<ColorRGBu8> for ColorRGBu8 {
    type Output = ColorRGBu8;

    fn add(self, rhs: ColorRGBu8) -> Self::Output{
        return ColorRGBu8 {
            r: self.r.saturating_add(rhs.r),
            g: self.g.saturating_add(rhs.g),
            b: self.b.saturating_add(rhs.b),
        }
    }
}

impl std::ops::Sub<ColorRGBu8> for ColorRGBu8 {
    type Output = ColorRGBu8;

    fn sub(self, rhs: ColorRGBu8) -> Self::Output{
        return ColorRGBu8 {
            r: self.r.saturating_sub(rhs.r),
            g: self.g.saturating_sub(rhs.g),
            b: self.b.saturating_sub(rhs.b),
        }
    }
}

impl From<u32> for ColorRGBu8 {
    fn from(value: u32) -> Self {
        return ColorRGBu8 {
            r: ((value & 0xff0000) >> 16) as u8,
            g: ((value & 0x00ff00) >> 8) as u8,
            b: (value & 0x0000ff) as u8,
        }
    }
}

impl From<ColorRGBf32> for ColorRGBu8 {
    fn from(value: ColorRGBf32) -> Self {
        return ColorRGBu8 {
            r: f32::clamp(value.r * 255.0,0.0,255.0) as u8,
            g: f32::clamp(value.g * 255.0,0.0,255.0) as u8,
            b: f32::clamp(value.b * 255.0,0.0,255.0) as u8,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorRGBf32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl std::ops::AddAssign<ColorRGBf32> for ColorRGBf32 {
    fn add_assign(&mut self, rhs: ColorRGBf32) {
        self.r = self.r + rhs.r;
        self.g = self.g + rhs.g;
        self.b = self.b + rhs.b;
    }
}

impl std::ops::SubAssign<ColorRGBf32> for ColorRGBf32 {
    fn sub_assign(&mut self, rhs: ColorRGBf32) {
        self.r = f32::max(self.r - rhs.r,0.0);
        self.g = f32::max(self.g - rhs.g,0.0);
        self.b = f32::max(self.b - rhs.b,0.0);
    }
}

impl std::ops::Add<ColorRGBf32> for ColorRGBf32 {
    type Output = ColorRGBf32;

    fn add(self, rhs: ColorRGBf32) -> Self::Output{
        return ColorRGBf32 {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl std::ops::Sub<ColorRGBf32> for ColorRGBf32 {
    type Output = ColorRGBf32;

    fn sub(self, rhs: ColorRGBf32) -> Self::Output{
        return ColorRGBf32 {
            r: f32::max(self.r - rhs.r,0.0),
            g: f32::max(self.g - rhs.g,0.0),
            b: f32::max(self.b - rhs.b,0.0),
        }
    }
}

impl Default for ColorRGBf32{
    fn default() -> Self {
        return ColorRGBf32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

impl From<ColorRGBu8> for ColorRGBf32 {
    fn from(value: ColorRGBu8) -> Self {
        return ColorRGBf32 {
            r: (value.r as f32 / 255.0),
            g: (value.g as f32 / 255.0),
            b: (value.b as f32 / 255.0),
        }
    }
}

pub enum ColorMode{
    Mode8bit,Mode24bit
}

pub struct Viewport<'a>{
    width:usize,
    height:usize,
    color_mode:ColorMode,
    buf:Vec<Vec<String>>,
    out:BufWriter<StdoutLock<'a>>
}

impl<'a> Viewport<'a> {
    pub fn new(w:usize,h:usize,mode:ColorMode) -> Self{
        let mut v = Viewport{
            width:w,
            height:h,
            color_mode:mode,
            buf:vec![],
            out:BufWriter::new(stdout().lock())
        };
        v.change_size(w,h);
        return v;
    }

    pub fn change_size(&mut self, w:usize, h:usize){
        self.buf = vec![vec![String::with_capacity(15);w];h];
        for y in 0..h{
            for x in 0..w{
                self.buf[y][x] = "\x1B[38;5;016mB\x1B[m".to_string();
            }
        }
        self.width = w;
        self.height = h;
    }

    pub fn clean(&mut self){
        self.out.write(b"\x1B[0;0H\x1B[c\x1B[?25l\x1B[2J").unwrap();
        self.out.flush().unwrap();
    }

    pub fn display(&mut self,t:&Vec<Vec<RenderChar>>){
        let buf = &mut self.buf;
        self.out.write(b"\x1B[0;0H\x1B[?25l").unwrap();
        for y in 0..self.height{
            if let Some(v) = t.get(y){
                for x in 0..self.width{
                    if let Some(rch) = v.get(x){
                        let color:Color8bit = rch.color.into();
                        let s = &mut buf[y][x];
                        let c1 = from_digit((color.data / 100) as u32,10).unwrap();
                        let c2 = from_digit(((color.data / 10) % 10) as u32,10).unwrap();
                        let c3 = from_digit((color.data % 10) as u32,10).unwrap();
                        let mut c_string = String::with_capacity(3);
                        c_string.push(c1);
                        c_string.push(c2);
                        c_string.push(c3);

                        s.replace_range((Included(7),Excluded(10)),c_string.as_str());
                        s.replace_range((Included(11), Included(11)), String::from(rch.c).as_str());
                        self.out.write(s.as_bytes()).unwrap();
                    }
                    else{
                        break;
                    }
                }
                self.out.write(b"\n").unwrap();
            }
            else{
                break;
            }
        }
        self.out.flush().unwrap();
    }

    pub fn width(&self) -> usize{
        return self.width;
    }

    pub fn height(&self) -> usize{
        return self.height;
    }
}

pub struct EventHandler<'a>{
    updates:Vec<Box<dyn Fn(&Duration) + 'a>>,
    render:Option<Box<dyn Fn(&AsciaEngine) -> Vec<Vec<RenderChar>> + 'a>>
}

impl<'a> Default for EventHandler<'a>{
    fn default() -> Self {
        return EventHandler{
            updates: vec![],
            render: None,
        }
    }
}

impl<'a> EventHandler<'a>{

    pub fn register_update_event<E:Entity + ?Sized + 'a>(&mut self, entity:Rc<RefCell<E>>){
        self.updates.push(Box::new(move |duration:&Duration| {
            entity.borrow_mut().update(duration);
        }));
    }

    pub fn set_render_event<C:CameraEntity + ?Sized + 'a>(&mut self,camera:Rc<RefCell<C>>){
        self.render = Some(Box::new(move |engine:&AsciaEngine| {
            return camera.borrow_mut().render(engine);
        }));
    }

    pub fn update(&self,duration:&Duration){
        for f in &self.updates{
            f(duration);
        }
    }

    pub fn render(&self,engine:&AsciaEngine) -> Vec<Vec<RenderChar>>{
        if let Some(r) = &self.render{
            return r(engine);
        }
        return vec![vec![RenderChar::default();engine.viewport.width];engine.viewport.height];
    }
}

pub struct SimpleEntity<'a>{
    pub objn:Rc<RefCell<ObjectNode>>,
    update:Box<dyn Fn(RefMut<ObjectNode>,&Duration) + 'a>
}

impl<'a> SimpleEntity<'a> {
    pub fn new(objn:ObjectNode) -> SimpleEntity<'a> {
        return SimpleEntity{
            objn:Rc::new(RefCell::new(objn)),
            update:Box::new(|_o:RefMut<ObjectNode>,_i:&Duration|{})
        };
    }

    pub fn register_update_fn<F:Fn(RefMut<ObjectNode>,&Duration) + 'a>(&mut self,f:F){
        self.update = Box::new(f);
    }
    pub fn connect(&self,parent:&SimpleEntity){
        ObjectNode::connect(&parent.objn,&self.objn);
    }

}

impl<'a> Entity for SimpleEntity<'a> {
    fn repr<'b>(&'b self) -> &'b Rc<RefCell<ObjectNode>> {
        return &self.objn;
    }
    fn update(&mut self,engine_time:&Duration) {
        (*(self).update)(self.repr().borrow_mut() ,engine_time);
    }
}

#[derive(PartialEq)]
pub enum EngineStatus{
    Waiting,
    UpdatingEntities,
    UpdatingNodes,
    Rendering
}

pub struct AsciaEngine<'a>{
    pub genesis:Rc<RefCell<ObjectNode>>,
    pub lights:Vec<Rc<RefCell<Light>>>,
    pub particles:Vec<CParticle>,
    pub viewport:Viewport<'a>,
    pub event_handler:EventHandler<'a>,
    pub wgpu_daq: Option<(wgpu::Device,wgpu::Queue)>,
    engine_status:EngineStatus,
    engine_time:Duration,
    engine_started:Instant
}

impl<'a> AsciaEngine<'a>{
    pub fn new(width:usize,height:usize) -> Self{
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let limits = wgpu::Limits::default();
        let mut wgpu_daq:Option<(wgpu::Device,wgpu::Queue)> = None;
        /*
        limits.max_buffer_size = 268435456u64 * 16u64;
        limits.max_storage_buffer_binding_size = u32::MAX;
         */
        if let Some(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())){
            if let Ok(daq) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor{
                label: None,
                features: wgpu::Features::SHADER_F16,
                limits: limits,
            }, None)){
                wgpu_daq = Some(daq);
            }
        }

        return AsciaEngine{
            genesis:Rc::new(RefCell::new(ObjectNode::new())),
            lights:vec![],
            particles:vec![],
            viewport:Viewport::new(width,height,ColorMode::Mode8bit),
            event_handler:EventHandler::default(),
            wgpu_daq: wgpu_daq,
            engine_status:Waiting,
            engine_time:Duration::ZERO,
            engine_started:Instant::now()
        }
    }

    pub fn update(&mut self){
        if self.engine_status != Waiting {
            return;
        }
        self.engine_status = UpdatingEntities;
        self.engine_time = self.engine_started.elapsed();
        self.event_handler.update(&self.engine_time);
        self.engine_status = UpdatingNodes;
        self.genesis.borrow_mut().update_global_properties_recursive();
        self.engine_status = Waiting;
    }

    pub fn render(&mut self){
        if self.engine_status != Waiting {
            return;
        }
        self.engine_status = Rendering;
        self.viewport.display(&self.event_handler.render(self));
        self.engine_status = Waiting;
    }

    pub fn engine_time(&self) -> Duration{
        return self.engine_time.clone();
    }
}