use std::cell::{RefCell};
use std::collections::{HashMap, VecDeque};
use std::io::{BufWriter, stdout, StdoutLock, Write};
use std::marker::PhantomData;
use std::rc::{Rc};
use std::time::{Duration, Instant};
use crate::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use crate::ascia::color::{ColorANSI256, ColorRGBf32, ColorRGBu8};
use crate::ascia::lights::PointLight;
use crate::ascia::math::{AABB3D, Matrix33, Quaternion, Vec2, Vec3};

#[cfg(feature = "wgpu")]
use crate::ascia::camera_wgpu::GPUWrapper;

pub trait AsciaEnvironment where Self: 'static{
    type PolygonMaterials: MaterialCollection<Polygon<Self, Global>> + Clone;
    type CParticleMaterials: MaterialCollection<CParticle<Self, Global>> + Clone;
    type Cameras: CameraDispatcher<Self>;
    type Lights: LightDispatcher<Self>;
    type ObjectNodeAttributes: ObjectNodeAttributeDispatcher<Self>;
}

pub trait ObjectNodeAttribute<E: AsciaEnvironment + ?Sized>{
    fn make_attribute_enum(self) -> E::ObjectNodeAttributes where E::ObjectNodeAttributes: From<Self>, Self: Sized{
        E::ObjectNodeAttributes::from(self)
    }
}

pub trait Material<E: AsciaEnvironment + ?Sized, CA: Camera<E> + ?Sized, RT: RaytracingTarget<0>>: Clone{
    type Output;
    fn calc_color<'a>(&self, intersection: &RT::Intersection<'a>, engine: &AsciaEngine<E>, camera:&CA, camera_node: &ObjectNode<E,Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output;
}

pub trait MaterialCollection<RT: RaytracingTarget<0>>: Default{}
pub trait MaterialDispatcher<E: AsciaEnvironment + ?Sized, RT: RaytracingTarget<0>>{
    fn calc_color(&self, intersection: &RT::Intersection<'_>, engine: &AsciaEngine<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> (ColorRGBf32, u32);
}

pub trait Camera<E: AsciaEnvironment + ?Sized>: ObjectNodeAttribute<E> {
    fn render(&self, camera_node: &ObjectNode<E, Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>>;
    fn make_camera_dispatcher(self) -> E::Cameras where E::Cameras: From<Self>, Self: Sized{
        E::Cameras::from(self)
    }
}
pub trait CameraDispatcher<E: AsciaEnvironment + ?Sized>: Default{
    fn render(&self, camera_node: &ObjectNode<E, Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>>;
    fn make_attribute_enum(self) -> E::ObjectNodeAttributes where E::ObjectNodeAttributes: From<Self>, Self:Sized{
        E::ObjectNodeAttributes::from(self)
    }
}

pub trait Light<E: AsciaEnvironment + ?Sized>: ObjectNodeAttribute<E> {
    fn ray(&self, light_node: &ObjectNode<E, Global>, to: &Vec3) -> ColorRGBf32;
    fn make_light_dispatcher(self) -> E::Lights where E::Lights: From<Self>, Self: Sized{
        E::Lights::from(self)
    }
}

pub trait LightDispatcher<E: AsciaEnvironment + ?Sized>: Default{
    fn ray(&self, light_node: &ObjectNode<E, Global>, to: &Vec3) -> ColorRGBf32;
    fn make_attribute_enum(self) -> E::ObjectNodeAttributes where E::ObjectNodeAttributes: From<Self>, Self:Sized{
        E::ObjectNodeAttributes::from(self)
    }
}

pub trait ObjectNodeAttributeDispatcher<E: AsciaEnvironment + ?Sized>{
    fn make_shared(self) -> Rc<RefCell<Option<Self>>> where Self: Sized;
    fn camera(&self) -> Option<&E::Cameras>;
    fn light(&self) -> Option<&E::Lights>;
}

pub struct PresetAsciaEnvironment{}

#[derive(Copy, Clone)]
pub enum PresetPolygonMaterial {
    FlatMaterial(FlatMaterial),
    LambertMaterial(LambertMaterial),
    LambertWithShadowMaterial(LambertWithShadowMaterial),
}

impl Default for PresetPolygonMaterial {
    fn default() -> Self {
        PresetPolygonMaterial::FlatMaterial(FlatMaterial::default())
    }
}

impl<E: AsciaEnvironment> MaterialCollection<Polygon<E, Global>> for PresetPolygonMaterial {}

#[cfg(not(feature = "wgpu"))]
pub enum PresetCamera<E: AsciaEnvironment<PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>>{
    SimpleCamera(SimpleCamera<E>),
    SimpleBVHCamera(SimpleBVHCamera<E>),
}

#[cfg(feature = "wgpu")]
pub enum PresetCamera<E: AsciaEnvironment<PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>>{
    SimpleCamera(SimpleCamera<E>),
    SimpleBVHCamera(SimpleBVHCamera<E>),
    SimpleCameraGPU(GPUWrapper<E, SimpleCamera<E>>),
    SimpleBVHCameraGPU(GPUWrapper<E, SimpleBVHCamera<E>>)
}

impl<E: AsciaEnvironment<PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> Default for PresetCamera<E> {
    fn default() -> Self {
        PresetCamera::SimpleCamera(SimpleCamera::default())
    }
}

#[cfg(not(feature = "wgpu"))]
impl<E: AsciaEnvironment<Cameras = PresetCamera<E>, PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> CameraDispatcher<E> for PresetCamera<E>{
    fn render(&self, camera_node: &ObjectNode<E,Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>> {
        match self {
            PresetCamera::SimpleCamera(c) => {c.render(camera_node, engine)}
            PresetCamera::SimpleBVHCamera(c) => {c.render(camera_node, engine)}
        }
    }
}

#[cfg(feature = "wgpu")]
impl<E: AsciaEnvironment<Cameras = PresetCamera<E>, PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> CameraDispatcher<E> for PresetCamera<E>{
    fn render(&self, camera_node: &ObjectNode<E,Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>> {
        match self {
            PresetCamera::SimpleCamera(c) => {c.render(camera_node, engine)}
            PresetCamera::SimpleBVHCamera(c) => {c.render(camera_node, engine)}
            PresetCamera::SimpleCameraGPU(c) => {c.render(camera_node, engine)}
            PresetCamera::SimpleBVHCameraGPU(c) => {c.render(camera_node, engine)}
        }
    }
}

pub enum PresetLight{
    PointLight(PointLight)
}

impl Default for PresetLight{
    fn default() -> Self {
        PresetLight::PointLight(PointLight::default())
    }
}

impl<E: AsciaEnvironment> LightDispatcher<E> for PresetLight{
    fn ray(&self, light_node: &ObjectNode<E, Global>, to: &Vec3) -> ColorRGBf32 {
        match self {
            PresetLight::PointLight(l) => { l.ray(light_node, to) }
        }
    }
}

pub enum PresetObjectNodeAttributeDispatcher<E: AsciaEnvironment>{
    Camera(E::Cameras),
    Light(E::Lights)
}

impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Cameras = PresetCamera<E>, Lights = PresetLight>> From<PresetCamera<E>> for PresetObjectNodeAttributeDispatcher<E>{
    fn from(value: PresetCamera<E>) -> Self {
        PresetObjectNodeAttributeDispatcher::Camera(value)
    }
}

impl<E: AsciaEnvironment<Lights = PresetLight>> From<PresetLight> for PresetObjectNodeAttributeDispatcher<E>{
    fn from(value: PresetLight) -> Self {
        PresetObjectNodeAttributeDispatcher::Light(value)
    }
}

impl<E:AsciaEnvironment> ObjectNodeAttributeDispatcher<E> for PresetObjectNodeAttributeDispatcher<E>{
    fn make_shared(self) -> Rc<RefCell<Option<Self>>> where Self: Sized{
        Rc::new(RefCell::new(Some(self)))
    }

    fn camera(&self) -> Option<&E::Cameras> {
        if let Self::Camera(c) = self{
            Some(c)
        }
        else{
            None
        }
    }

    fn light(&self) -> Option<&E::Lights> {
        if let Self::Light(l) = self{
            Some(l)
        }
        else{
            None
        }
    }
}

#[derive(Copy, Clone)]
pub enum PresetCParticleMaterial {
    FlatMaterial(FlatMaterial),
    LambertMaterial(LambertMaterial),
    LambertWithShadowMaterial(LambertWithShadowMaterial),
}

impl Default for PresetCParticleMaterial {
    fn default() -> Self {
        PresetCParticleMaterial::FlatMaterial(FlatMaterial::default())
    }
}

impl<E: AsciaEnvironment> MaterialCollection<CParticle<E, Global>> for PresetCParticleMaterial {}

impl AsciaEnvironment for PresetAsciaEnvironment{
    type PolygonMaterials = PresetPolygonMaterial;
    type CParticleMaterials = PresetCParticleMaterial;
    type Cameras = PresetCamera<Self>;
    type Lights = PresetLight;
    type ObjectNodeAttributes = PresetObjectNodeAttributeDispatcher<Self>;
}

pub trait CoordinateType where Self: 'static{}

pub struct Local{}

impl CoordinateType for Local{}

pub struct Global{}

impl CoordinateType for Global{}

#[derive(Debug, Copy, Clone)]
pub struct FlatMaterial{
    pub color: ColorRGBf32,
    pub priority: u32,
}

impl Default for FlatMaterial{
    fn default() -> Self {
        return FlatMaterial{
            color: ColorRGBf32{
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            priority: 0,
        };
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LambertMaterial{
    pub color: ColorRGBf32,
    pub priority: u32,
}

impl Default for LambertMaterial{
    fn default() -> Self {
        return LambertMaterial{
            color: ColorRGBf32{
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            priority: 0,
        };
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LambertWithShadowMaterial{
    pub color: ColorRGBf32,
    pub priority: u32,
}
impl Default for LambertWithShadowMaterial{
    fn default() -> Self {
        return LambertWithShadowMaterial{
            color: ColorRGBf32{
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            priority: 0,
        };
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray{
    pub position:Vec3, // TODO CoordinateType
    pub direction:Vec3,
}

impl Ray{
    pub fn project<'a, const RC: usize, T: RaytracingTarget<RC>, F: Fn(&T::Intersection<'a>) -> bool>(&self, target: &'a T, exclude_cond: &F) -> Option<T::Intersection<'a>>{
        return target.project_by(self, exclude_cond);
    }
}

pub trait RayIntersection{
    fn position(&self) -> Vec3;
    fn ray(&self) -> Ray;
    fn depth(&self) -> f32;
}

#[derive(Copy)]
pub struct PolygonRayIntersection<'a, E: AsciaEnvironment + ?Sized + 'static, CO:CoordinateType>{
    pub polygon: &'a Polygon<E, CO>,
    pub depth:f32,
    pub position_on_polygon:Vec2,
    pub position:Vec3,
    pub ray: Ray,
    pub normal: Vec3
}

impl<'a, E:AsciaEnvironment + ?Sized, CO:CoordinateType> Clone for PolygonRayIntersection<'a, E, CO>{
    fn clone(&self) -> Self {
        return PolygonRayIntersection{
            polygon: self.polygon,
            depth: self.depth,
            position_on_polygon: self.position_on_polygon,
            position: self.position,
            ray: self.ray,
            normal: self.normal,
        }
    }
}

impl<'a, E:AsciaEnvironment + ?Sized, CO:CoordinateType> RayIntersection for PolygonRayIntersection<'a, E, CO>{
    fn position(&self) -> Vec3 {
        return self.position;
    }

    fn ray(&self) -> Ray {
        return self.ray;
    }

    fn depth(&self) -> f32 {
        return self.depth;
    }
}

pub struct CParticleRayIntersection<'a, E: AsciaEnvironment + ?Sized, CO:CoordinateType>{
    pub particle: &'a CParticle<E, CO>,
    pub depth:f32,
    pub position:Vec3,
    pub ray: Ray,
    pub distance: f32,
    _ph: PhantomData<E>
}

impl<'a, E: AsciaEnvironment + ?Sized, C:CoordinateType> Clone for CParticleRayIntersection<'a, E, C>{
    fn clone(&self) -> Self {
        return CParticleRayIntersection{
            particle: self.particle,
            depth: self.depth,
            position: self.position,
            ray: self.ray,
            distance: self.distance,
            _ph: Default::default(),
        }
    }
}

impl<'a, E: AsciaEnvironment + ?Sized, C:CoordinateType> RayIntersection for CParticleRayIntersection<'a, E, C>{
    fn position(&self) -> Vec3 {
        return self.position;
    }

    fn ray(&self) -> Ray {
        return self.ray;
    }

    fn depth(&self) -> f32 {
        return self.depth;
    }
}


#[derive(Copy, Clone)]
pub struct AABB3DRayIntersection{
    pub depth:f32,
    pub position:Vec3,
    pub ray: Ray
}

impl<'a> RayIntersection for AABB3DRayIntersection{
    fn position(&self) -> Vec3 {
        return self.position;
    }

    fn ray(&self) -> Ray {
        return self.ray;
    }

    fn depth(&self) -> f32 {
        return self.depth;
    }

}

pub trait RaytracingTarget<const RECURSION_COUNT: usize>{
    type Intersection<'b>: RayIntersection where Self: 'b;
    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>>;
}



#[derive(Debug, Copy)]
pub struct Polygon<E: AsciaEnvironment + ?Sized + 'static, CO: CoordinateType>{
    pub poses:Matrix33,
    pub material: E::PolygonMaterials,
    pub _ph: PhantomData<CO>
}

impl<E:AsciaEnvironment + ?Sized + 'static, CO:CoordinateType> RaytracingTarget<0> for Polygon<E, CO>{
    type Intersection<'a> = PolygonRayIntersection<'a, E, CO>;

    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>> {
        let m2 = Matrix33{
            v1: self.poses.v1 - ray.position,
            v2: self.poses.v2 - ray.position,
            v3: self.poses.v3 - ray.position,
        };
        if let Some(m3) = m2.inverse(){
            let v1 = m3 * (ray.direction);
            let psy = 1.0 / (v1.x + v1.y + v1.z);
            if psy != f32::INFINITY && 0.0 < psy{
                let v2 = v1 * psy;
                if (0.0 <= v2.x && v2.x <= 1.0) && (0.0 <= v2.y && v2.y <= 1.0) && (0.0 <= v2.z && v2.z <= 1.0){
                    let depth = psy * ray.direction.norm();
                    if depth > 0.0{
                        let i = Self::Intersection{
                            polygon: self,
                            position_on_polygon: Vec2{
                                x:v2.y,
                                y:v2.z
                            },
                            depth: depth,
                            position: self.poses.v1 + v2.y * (self.poses.v2 - self.poses.v1) + v2.z * (self.poses.v3 - self.poses.v1),
                            ray: ray.clone(),
                            normal: ((self.poses.v2 - self.poses.v1) ^ (self.poses.v3 - self.poses.v1)).normalize()
                        };
                        return if exclude_cond(&i) { None } else { Some(i) };
                    }
                }
            }
        }
        return None;
    }
}

impl<E:AsciaEnvironment + ?Sized, C:CoordinateType> RaytracingTarget<0> for CParticle<E, C>{
    type Intersection<'a> = CParticleRayIntersection<'a, E, C>;

    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>> {
        let a = self.position - ray.position;
        let k = (ray.direction * a) / (ray.direction * ray.direction);
        let d = (a - k * ray.direction).norm();
        match self.mode {
            CParticleMode::SPHERE => {
                if d < self.threshold{
                    let i = CParticleRayIntersection{
                        particle: self,
                        depth: k * ray.direction.norm(),
                        position: ray.position + ray.direction * (k * ray.direction.norm()),
                        ray: ray.clone(),
                        distance: d,
                        _ph: Default::default(),
                    };
                    return if exclude_cond(&i) { None } else { Some(i) };
                }
            }
            CParticleMode::ARG => {
                if f32::sqrt((a * a) / (ray.direction * ray.direction)) * f32::cos(self.threshold) <= k{
                    let i = CParticleRayIntersection{
                        particle: self,
                        depth: k * ray.direction.norm(),
                        position: ray.position + ray.direction * (k * ray.direction.norm()),
                        ray: ray.clone(),
                        distance: d,
                        _ph: Default::default(),
                    };
                    return if exclude_cond(&i) { None } else { Some(i) };
                }
            }
        }
        return None;
    }
}

impl RaytracingTarget<0> for AABB3D{
    type Intersection<'a> = AABB3DRayIntersection;

    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>> {
        let v0 = Vec3{
            x: (self.a.x - ray.position.x) / ray.direction.x,
            y: (self.a.y - ray.position.y) / ray.direction.y,
            z: (self.a.z - ray.position.z) / ray.direction.z,
        };
        let v1 = Vec3{
            x: (self.b.x - ray.position.x) / ray.direction.x,
            y: (self.b.y - ray.position.y) / ray.direction.y,
            z: (self.b.z - ray.position.z) / ray.direction.z,
        };
        
        let v_min = Vec3{
            x: f32::min(v0.x, v1.x),
            y: f32::min(v0.y, v1.y),
            z: f32::min(v0.z, v1.z)
        };
        let v_max = Vec3{
            x: f32::max(v0.x, v1.x),
            y: f32::max(v0.y, v1.y),
            z: f32::max(v0.z, v1.z)
        };
        
        if f32::max(f32::max(v_min.x, v_min.y),v_min.z) <= f32::min(f32::min(v_max.x, v_max.y),v_max.z){
            let depth = f32::min(f32::min(v_min.x, v_min.y),v_min.z);
            let i = AABB3DRayIntersection{
                depth: depth,
                position: ray.position + ray.direction * depth,
                ray: ray.clone(),
            };
            return if exclude_cond(&i) { None } else { Some(i) };
        }
        
        return None;
    }
}

impl<T: RaytracingTarget<0>> RaytracingTarget<1> for [T] {
    type Intersection<'a> = T::Intersection<'a> where T: 'a;

    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>> {
        let mut nearest:Option<Self::Intersection<'a>> = None;
        for iter in self{
            if let Some(i) = iter.project_by(ray, exclude_cond){
                if let Some(j) = &nearest{
                    if i.depth() < j.depth(){
                        nearest = Some(i);
                    }
                }
                else{
                    nearest = Some(i);
                }
            }
        }
        return nearest;
    }

}



impl<T> RaytracingTarget<1> for Vec<T> where T: RaytracingTarget<0> {
    type Intersection<'a> = <T as RaytracingTarget<0>>::Intersection<'a> where T: 'a;

    fn project_by<'a, F: Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection<'a>>{
        let mut nearest:Option<Self::Intersection<'a>> = None;
        for iter in self{
            if let Some(i) = iter.project_by(ray, exclude_cond){
                if let Some(j) = &nearest{
                    if i.depth() < j.depth(){
                        nearest = Some(i);
                    }
                }
                else{
                    nearest = Some(i);
                }
            }
        }
        return nearest;
    }
}


impl<E: AsciaEnvironment, CO: CoordinateType> Clone for Polygon<E, CO>{
    fn clone(&self) -> Self {
        return Polygon{
            poses: self.poses,
            material: self.material.clone(),
            _ph: self._ph
        }
    }
}


impl<E:AsciaEnvironment,CO:CoordinateType> Polygon<E, CO> {
    pub fn new(p1:&Vec3, p2:&Vec3, p3:&Vec3) -> Self{
        return Polygon {
            poses:Matrix33{
                v1:p1.clone(),
                v2:p2.clone(),
                v3:p3.clone()
            },
            material: E::PolygonMaterials::default(),
            _ph: Default::default()
        }
    }
    
    pub fn aabb(&self) -> AABB3D{
        return AABB3D::generate_3(&self.poses.v1, &self.poses.v2, &self.poses.v3);
    }
}

pub struct ObjectNode<E: AsciaEnvironment + ?Sized + 'static,CO:CoordinateType>{
    pub tag: String,
    pub attribute: Rc<RefCell<Option<E::ObjectNodeAttributes>>>,
    pub position: Vec3,
    pub direction: Quaternion,
    pub polygons: Vec<Polygon<E, CO>>,
    pub c_particles: Vec<CParticle<E, CO>>,
    pub children: HashMap<String, ObjectNode<E, CO>>,
}

#[derive(Copy)]
#[repr(C)]
pub struct CParticle<E: AsciaEnvironment + ?Sized + 'static, C: CoordinateType> {
    pub position:Vec3,
    pub velocity:Vec3,
    pub c:char,
    pub threshold:f32,
    pub mode:CParticleMode,
    pub material: E::CParticleMaterials,
    pub _ph: PhantomData<C>
}

impl<E: AsciaEnvironment, C: CoordinateType> Clone for CParticle<E, C>{
    fn clone(&self) -> Self {
        return CParticle{
            position: self.position,
            velocity: self.velocity,
            c: self.c,
            threshold: self.threshold,
            mode: self.mode,
            material: self.material.clone(),
            _ph: Default::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum CParticleMode{
    SPHERE = 0u32,
    ARG = 1u32
}

pub struct ObjectNodeIter<'a, E:AsciaEnvironment, CO: CoordinateType>{
    prior: Option<&'a ObjectNode<E, CO>>,
    stack: VecDeque<std::collections::hash_map::Iter<'a,String, ObjectNode<E, CO>>>,
}

impl<'a, E:AsciaEnvironment, CO: CoordinateType> Iterator for ObjectNodeIter<'a, E, CO>{
    type Item = &'a ObjectNode<E, CO>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.prior{
            self.stack.push_back(root.children.iter());
            self.prior = None;
            return Some(root);
        }
        while let Some(iter) = self.stack.back_mut(){
            if let Some(now) = iter.next(){
                self.stack.push_back(now.1.children.iter());
                return Some(now.1);
            }
            else{
                self.stack.pop_back();
            }
        }
        return None;
    }
}

impl<E:AsciaEnvironment, C:CoordinateType> ObjectNode<E, C>{
    pub fn position(&self) -> Vec3{
        return self.position.clone();
    }

    pub fn direction(&self) -> Quaternion{
        return self.direction.clone();
    }

    pub fn add_child(&mut self, child: ObjectNode<E, C>){
        self.children.insert(child.tag.clone(), child);
    }

    pub fn remove_child(&mut self, tag: String){
        self.children.remove(&tag);
    }

    pub fn child(&self, tag: &str) -> Option<&ObjectNode<E, C>>{
        return self.children.get(&tag.to_string());
    }

    pub fn child_mut(&mut self, tag: &str) -> Option<&mut ObjectNode<E, C>>{
        return self.children.get_mut(&tag.to_string());
    }

    pub fn iter(&self) -> ObjectNodeIter<E, C>{
        let stack = VecDeque::new();
        return ObjectNodeIter{
            prior: Some(self),
            stack: stack
        };
    }
}

impl<E:AsciaEnvironment> ObjectNode<E, Local>{
    pub fn new(tag: &str) -> Self{
        return ObjectNode::from(tag,vec![]);
    }
    pub fn from(tag: &str, polygons:Vec<Polygon<E, Local>>) -> Self {
        return ObjectNode{
            tag: tag.to_string(),
            attribute: Rc::new(RefCell::new(None)),
            position: Vec3::default(),
            direction: Quaternion::default(),
            polygons: polygons,
            c_particles: vec![],
            children: Default::default()
        };
    }
    pub fn generate_global_nodes(&self) -> ObjectNode<E, Global>{
        let mut iter = self.iter();
        let mut stack_global:VecDeque<ObjectNode<E, Global>> = VecDeque::new();
        while let Some(now) = iter.next(){
            while stack_global.len() + 1 > iter.stack.len(){
                let node = stack_global.pop_back();
                stack_global.back_mut().unwrap().add_child(node.unwrap());
            }

            let parent_position = if stack_global.is_empty() {
                Vec3::default()
            } else {
                stack_global.back().unwrap().position
            };
            let parent_direction = if stack_global.is_empty() {
                Quaternion::default()
            } else {
                stack_global.back().unwrap().direction
            };

            let mut child = ObjectNode{
                tag: now.tag.clone(),
                attribute: now.attribute.clone(),
                position: parent_position + parent_direction.rotate(&now.position),
                direction: parent_direction * now.direction,
                polygons: Vec::with_capacity(now.polygons.len()),
                c_particles: Vec::with_capacity(now.c_particles.len()),
                children: Default::default(),
            };
            for p in &now.polygons{
                child.polygons.push(Polygon{
                    poses: Matrix33 {
                        v1: child.position + child.direction.rotate(&p.poses.v1),
                        v2: child.position + child.direction.rotate(&p.poses.v2),
                        v3: child.position + child.direction.rotate(&p.poses.v3),
                    },
                    material: p.material.clone(),
                    _ph: Default::default(),
                });
            }
            for p in &now.c_particles {
                child.c_particles.push(CParticle {
                    position: child.position + child.direction.rotate(&p.position),
                    velocity: p.velocity,
                    c: p.c,
                    threshold: p.threshold,
                    mode: p.mode,
                    material: p.material.clone(),
                    _ph: Default::default(),
                });
            }
            stack_global.push_back(child);
        }
        while stack_global.len() > 1{
            let node = stack_global.pop_back();
            stack_global.back_mut().unwrap().add_child(node.unwrap());
        }
        return stack_global.pop_back().unwrap();
    }
}

#[cfg(test)]
mod tests{
    use std::cell::RefCell;
    use std::f32::consts::PI;
    use std::rc::Rc;
    use crate::ascia::core::{Local, ObjectNode, Polygon, PresetAsciaEnvironment};
    use crate::ascia::math::{Quaternion, Vec3};

    #[test]
    fn test_generate_global_nodes(){
        let a_pos_local = Vec3{
            x: 0.0,
            y: 2.0,
            z: 0.0,
        };
        let a_dir_local = Quaternion::new(
            &Vec3{
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }, PI / 2.0, 1.0
        );
        let b_pos_local = Vec3{
            x: 2.0,
            y: 0.0,
            z: 0.0,
        };
        let c_pos_local = Vec3{
            x: 2.0,
            y: 3.0,
            z: 4.0,
        };
        let c_dir_local = Quaternion::new(&Vec3{
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }, PI * 0.5, 1.0);
        let d_pos_local = Vec3{
            x: 2.0,
            y: -2.0,
            z: 3.0,
        };
        let mut a: ObjectNode<PresetAsciaEnvironment, Local> = ObjectNode{
            tag: "a".to_string(),
            attribute: Rc::new(RefCell::new(None)),
            position: a_pos_local,
            direction: a_dir_local,
            polygons: vec![
                Polygon::new(&Vec3{
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    &Vec3{
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    &Vec3{
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    })
            ],
            c_particles: vec![],
            children: Default::default(),
        };
        let b: ObjectNode<PresetAsciaEnvironment, Local> = ObjectNode{
            tag: "b".to_string(),
            attribute: Rc::new(RefCell::new(None)),
            position: b_pos_local,
            direction: Default::default(),
            polygons: vec![
                Polygon::new(&Vec3{
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        &Vec3{
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        &Vec3{
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        })
            ],
            c_particles: vec![],
            children: Default::default(),
        };
        let mut c: ObjectNode<PresetAsciaEnvironment,Local> = ObjectNode{
            tag: "c".to_string(),
            attribute: Rc::new(RefCell::new(None)),
            position: c_pos_local,
            direction: c_dir_local,
            polygons: vec![
                Polygon::new(
                    &Vec3{
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        &Vec3{
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                        &Vec3{
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        })
            ],
            c_particles: vec![],
            children: Default::default(),
        };
        let d: ObjectNode<PresetAsciaEnvironment, Local> = ObjectNode{
            tag: "d".to_string(),
            attribute: Rc::new(RefCell::new(None)),
            position: d_pos_local,
            direction: Default::default(),
            polygons: vec![
                Polygon::new(
                    &Vec3{
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    &Vec3{
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    &Vec3{
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    })
            ],
            c_particles: vec![],
            children: Default::default(),
        };
        a.add_child(b);
        c.add_child(d);
        a.add_child(c);
        let a_global = a.generate_global_nodes();
        let b_global = a_global.children.get(&"b".to_string()).unwrap();
        let c_global = a_global.children.get(&"c".to_string()).unwrap();
        let d_global = c_global.children.get(&"d".to_string()).unwrap();
        assert_eq!(b_global.position, a_pos_local + a_dir_local.rotate(&b_pos_local));
        assert_eq!(b_global.direction, a_dir_local);
        assert_eq!(c_global.position, a_pos_local + a_dir_local.rotate(&c_pos_local));
        assert_eq!(c_global.direction,  a_dir_local * c_dir_local);
        assert_eq!(d_global.position, c_global.position + c_global.direction.rotate(&d_pos_local));
    }
}

#[derive(Copy, Clone)]
pub struct RenderChar{
    pub c:char,
    pub color: ColorRGBu8
}

impl Default for RenderChar{
    fn default() -> Self {
        return RenderChar{
            c: ' ',
            color: ColorRGBu8::default(),
        }
    }
}

pub trait Viewport{
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn change_size(&mut self, new_width:usize, new_height:usize);
    fn display(&mut self, cs:&Vec<Vec<RenderChar>>);
}

pub struct ViewportStdout<'a>{
    width:usize,
    height:usize,
    out:BufWriter<StdoutLock<'a>>
}

impl<'a> Viewport for ViewportStdout<'a>{
    fn width(&self) -> usize {
        return self.width;
    }

    fn height(&self) -> usize {
        return self.height;
    }

    fn change_size(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
    }
    
    fn display(&mut self, cs: &Vec<Vec<RenderChar>>) {
        let mut out = vec![];
        out.extend_from_slice(b"\x1B[0;0H");

        for line in cs{
            out.extend_from_slice(b"\x1B[?25l");
            for rc in line{
                let color: ColorANSI256 = rc.color.into();
                out.extend_from_slice(format!("\x1B[38;5;{}m{}\x1B[m", color.data, rc.c).as_ref());
            }
            out.extend_from_slice(b"\n");
        }

        out.extend_from_slice(b"\x1B[?25h");
        self.out.write(&out).unwrap();
        self.out.flush().unwrap();
    }
}

impl<'a> ViewportStdout<'a> {
    pub fn new(w:usize,h:usize) -> Self{
        let mut v = ViewportStdout {
            width:w,
            height:h,
            out:BufWriter::new(stdout().lock())
        };
        v.change_size(w,h);
        v.clean();
        return v;
    }
    
    pub fn clean(&mut self){
        self.out.write(b"\x1B[0;0H\x1B[c\x1B[?25l\x1B[2J").unwrap();
        self.out.flush().unwrap();
    }
}

pub struct AsciaEngine<E: AsciaEnvironment + ?Sized>{
    pub genesis_local: ObjectNode<E, Local>,
    pub genesis_global: ObjectNode<E, Global>,
    pub viewport: RefCell<Box<dyn Viewport>>,
    engine_time:Duration,
    engine_started:Instant
}

impl<E: AsciaEnvironment> AsciaEngine<E>{
    pub fn new(width:usize,height:usize) -> Self{
        return AsciaEngine{
            genesis_local: ObjectNode::new("genesis"),
            genesis_global: ObjectNode{
                tag: "genesis".to_string(),
                attribute: Rc::new(RefCell::new(None)),
                position: Default::default(),
                direction: Default::default(),
                polygons: vec![],
                c_particles: vec![],
                children: Default::default(),
            },
            viewport: RefCell::new(Box::new(ViewportStdout::new(width, height))),
            engine_time:Duration::ZERO,
            engine_started:Instant::now()
        }
    }

    pub fn render(&self, camera_node: &ObjectNode<E, Global>) -> Result<Vec<Vec<RenderChar>>, String>{
        if let Some(a) = &*camera_node.attribute.borrow(){
            if let Some(camera) = a.camera(){
                let result = camera.render(camera_node, self);
                self.viewport.borrow_mut().display(&result);
                return Ok(result);
            }
        }
        return Err("given node does not contain camera attribute".to_string());
    }

    pub fn engine_time(&self) -> Duration{
        return self.engine_time.clone();
    }

    pub fn engine_started_time(&self) -> Instant{
        return self.engine_started.clone();
    }
    
    pub fn sync_engine_time(&mut self){
        self.engine_time = self.engine_started.elapsed();
    }

    pub fn set_engine_time(&mut self, dur: &Duration){
        self.engine_time = dur.clone();
    }
    
    pub fn update_global_nodes(&mut self){
        self.genesis_global = self.genesis_local.generate_global_nodes();
    }
}
