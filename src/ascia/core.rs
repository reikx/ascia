use std::cell::{RefCell};
use std::char::{from_digit};
use std::collections::{HashMap, VecDeque};
use std::io::{BufWriter, stdout, StdoutLock, Write};
use std::marker::PhantomData;
use std::ops::Bound::{Excluded, Included};
use std::rc::{Rc};
use std::time::{Duration, Instant};
use crate::ascia::camera::{SimpleBVHCamera, SimpleCamera};
use crate::ascia::camera_gpu::GPUWrapper;
use crate::ascia::color::{Color8bit, ColorRGBf32, ColorRGBu8};
use crate::ascia::lights::PointLight;
use crate::ascia::math::{AABB3D, Matrix33, Quaternion, Vec2, Vec3};

pub trait AsciaEnvironment{
    type Materials;
    type Cameras;
    type ObjectNodeAttributes;
}

pub struct PresetAsciaEnvironment{
    
}

impl AsciaEnvironment for PresetAsciaEnvironment{
    type Materials = PresetMaterial;
    type Cameras = PresetCamera;
    type ObjectNodeAttributes = PresetObjectNodeAttribute;
}

pub trait CoordinateType{}

pub struct Local{}

impl CoordinateType for Local{}

pub struct Global{}

impl CoordinateType for Global{}

#[derive(Debug, Copy, Clone)]
pub struct Ray{
    pub position:Vec3, // TODO CoordinateType
    pub direction:Vec3,
}

impl Ray{
    pub fn project<T: RaytracingTarget, F: Fn(&T::Intersection) -> bool>(&self, target: T, exclude_cond: &F) -> Option<T::Intersection>{
        return target.project_by(self, exclude_cond);
    }
}

pub trait RayIntersection{
    fn position(&self) -> Vec3;
    fn ray(&self) -> Ray;
    fn depth(&self) -> f32;
}

#[derive(Copy)]
pub struct PolygonRayIntersection<'a, CO:CoordinateType>{
    pub polygon: &'a Polygon<CO>,
    pub depth:f32,
    pub position_on_polygon:Vec2,
    pub position:Vec3,
    pub ray: Ray,
    pub normal: Vec3
}

impl<'a, CO:CoordinateType> Clone for PolygonRayIntersection<'a, CO>{
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

impl<'a, CO:CoordinateType> RayIntersection for PolygonRayIntersection<'a, CO>{
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

pub struct CParticleRayIntersection<'a, CO:CoordinateType>{
    pub particle: &'a CParticle<CO>,
    pub depth:f32,
    pub position:Vec3,
    pub ray: Ray,
    pub distance: f32
}

impl<'a, C:CoordinateType> Clone for CParticleRayIntersection<'a, C>{
    fn clone(&self) -> Self {
        return CParticleRayIntersection{
            particle: self.particle,
            depth: self.depth,
            position: self.position,
            ray: self.ray,
            distance: self.distance,
        }
    }
}

impl<'a, C:CoordinateType> RayIntersection for CParticleRayIntersection<'a, C>{
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

pub trait RaytracingTarget {
    type Intersection: RayIntersection;
    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection>;
}

#[derive(Debug, Copy)]
pub struct Polygon<CO: CoordinateType>{
    pub poses:Matrix33,
    pub material: PresetMaterial,
    pub _ph: PhantomData<CO>
}



impl<'a, CO:CoordinateType> RaytracingTarget for &'a Polygon<CO>{
    type Intersection = PolygonRayIntersection<'a, CO>;

    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection> {
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

impl<'a, C:CoordinateType> RaytracingTarget for &'a CParticle<C>{
    type Intersection = CParticleRayIntersection<'a, C>;

    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection> {
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
                        distance: d
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
                        distance: d
                    };
                    return if exclude_cond(&i) { None } else { Some(i) };
                }
            }
        }
        return None;
    }
}

impl RaytracingTarget for AABB3D{
    type Intersection = AABB3DRayIntersection;

    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection> {
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

impl<'a,T: RaytracingTarget> RaytracingTarget for &'a [T] {
    type Intersection = T::Intersection;

    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection> {
        let mut nearest:Option<Self::Intersection> = None;
        for iter in *self{
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


impl<'a,T> RaytracingTarget for &'a Vec<T> where &'a T: RaytracingTarget {
    type Intersection = <&'a T as RaytracingTarget>::Intersection;

    fn project_by<F: Fn(&Self::Intersection) -> bool>(&self, ray: &Ray, exclude_cond: &F) -> Option<Self::Intersection>{
        let mut nearest:Option<Self::Intersection> = None;
        for iter in *self{
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


impl<CO: CoordinateType> Clone for Polygon<CO>{
    fn clone(&self) -> Self {
        return Polygon{
            poses: self.poses,
            material: self.material.clone(),
            _ph: self._ph
        }
    }
}


impl<CO:CoordinateType> Polygon<CO> {
    pub fn new(p1:&Vec3, p2:&Vec3, p3:&Vec3) -> Self{
        return Polygon {
            poses:Matrix33{
                v1:p1.clone(),
                v2:p2.clone(),
                v3:p3.clone()
            },
            material: PresetMaterial::Flat(Default::default()),
            _ph: Default::default()
        }
    }
    
    pub fn aabb(&self) -> AABB3D{
        return AABB3D::generate_3(&self.poses.v1, &self.poses.v2, &self.poses.v3);
    }
}

pub struct ObjectNode<CO:CoordinateType>{
    pub tag: String,
    pub attribute: Rc<RefCell<PresetObjectNodeAttribute>>,
    pub position: Vec3,
    pub direction: Quaternion,
    pub polygons: Vec<Polygon<CO>>,
    pub c_particles: Vec<CParticle<CO>>,
    pub children: HashMap<String, ObjectNode<CO>>,
}

pub enum PresetObjectNodeAttribute {
    Normal(),
    Camera(PresetCamera),
    Light(PresetLight)
}

pub trait ObjectNodeAttribute{
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>>;
}

pub trait Camera: ObjectNodeAttribute{
    fn render(&self, camera_node: &ObjectNode<Global>, engine: &AsciaEngine) -> Vec<Vec<RenderChar>>;
}

pub trait Light: ObjectNodeAttribute{
    fn ray(&self, light_node: &ObjectNode<Global>, to: &Vec3) -> ColorRGBf32;
}

pub trait Material<CA: Camera, RT: RaytracingTarget>: Clone{
    fn calc_color(&self, intersection: &RT::Intersection, engine: &AsciaEngine, camera:&CA, camera_node: &ObjectNode<Global>, global_polygons: &Vec<Polygon<Global>>) -> (ColorRGBf32, u32);

}

#[derive(Debug, Copy, Clone)]
pub enum PresetMaterial{
    Flat(FlatMaterial),
    Lambert(LambertMaterial),
    LambertWithShadow(LambertWithShadowMaterial),
}

#[derive(Debug, Copy, Clone)]
pub struct FlatMaterial{
    pub color: ColorRGBf32,
    pub priority: u32
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

impl<'a, CA: Camera> Material<CA, &'a CParticle<Global>> for PresetMaterial {
    fn calc_color(&self, intersection: &<&'a CParticle<Global> as RaytracingTarget>::Intersection, engine: &AsciaEngine, camera: &CA, camera_node: &ObjectNode<Global>, global_polygons: &Vec<Polygon<Global>>) -> (ColorRGBf32, u32) {
        match self {
            PresetMaterial::Flat(m) => {<FlatMaterial as Material<CA, &'a CParticle<Global>>>::calc_color(m, intersection, engine, camera, camera_node, global_polygons)}
            PresetMaterial::Lambert(m) => { panic!() }
            PresetMaterial::LambertWithShadow(m) => { panic!() }
        }
    }
}

impl<'a, CA: Camera> Material<CA, &'a Polygon<Global>> for PresetMaterial where LambertWithShadowMaterial: Material<CA, &'a Polygon<Global>>{
    fn calc_color(&self, intersection: &<&'a Polygon<Global> as RaytracingTarget>::Intersection, engine: &AsciaEngine, camera: &CA, camera_node: &ObjectNode<Global>, global_polygons: &Vec<Polygon<Global>>) -> (ColorRGBf32, u32) {
        match self {
            PresetMaterial::Flat(m) => {<FlatMaterial as Material<CA, &'a Polygon<Global>>>::calc_color(m, intersection, engine, camera, camera_node, global_polygons)}
            PresetMaterial::Lambert(m) => {<LambertMaterial as Material<CA, &'a Polygon<Global>>>::calc_color(m,intersection, engine, camera, camera_node, global_polygons)}
            PresetMaterial::LambertWithShadow(m) => {<LambertWithShadowMaterial as Material<CA, &'a Polygon<Global>>>::calc_color(m,intersection, engine, camera, camera_node, global_polygons)}
        }
    }
}

pub enum PresetCamera{
    Simple(SimpleCamera),
    SimpleGPU(GPUWrapper<SimpleCamera>),
    SimpleBVH(SimpleBVHCamera),
    SimpleBVHGPU(GPUWrapper<SimpleBVHCamera>),
}

pub enum PresetLight{
    Point(PointLight)
}

impl ObjectNodeAttribute for PresetCamera {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Camera(self)));
    }
}

impl Camera for PresetCamera{
    fn render(&self, node: &ObjectNode<Global>, engine: &AsciaEngine) -> Vec<Vec<RenderChar>> {
        match self {
            PresetCamera::Simple(camera) => {camera.render(node,engine)}
            PresetCamera::SimpleGPU(camera) => {camera.render(node,engine)}
            PresetCamera::SimpleBVH(camera) => {camera.render(node,engine)}
            PresetCamera::SimpleBVHGPU(camera) => {camera.render(node,engine)}
        }
    }
}

impl ObjectNodeAttribute for PresetLight {
    fn make_attribute_enum(self) -> Rc<RefCell<PresetObjectNodeAttribute>> {
        return Rc::new(RefCell::new(PresetObjectNodeAttribute::Light(self)));
    }
}

impl Light for PresetLight{
    fn ray(&self, node: &ObjectNode<Global>, to: &Vec3) -> ColorRGBf32 {
        match self{
            PresetLight::Point(light) => {light.ray(node, to)}
        }
    }
}

#[derive(Copy)]
#[repr(C)]
pub struct CParticle<C: CoordinateType> {
    pub position:Vec3,
    pub velocity:Vec3,
    pub color:ColorRGBf32,
    pub c:char,
    pub threshold:f32,
    pub mode:CParticleMode,
    pub _ph: PhantomData<C>
}

impl<C: CoordinateType> Clone for CParticle<C>{
    fn clone(&self) -> Self {
        return CParticle{
            position: self.position,
            velocity: self.velocity,
            color: self.color,
            c: self.c,
            threshold: self.threshold,
            mode: self.mode,
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

pub struct ObjectNodeIter<'a, CO: CoordinateType>{
    prior: Option<&'a ObjectNode<CO>>,
    stack: VecDeque<std::collections::hash_map::Iter<'a,String, ObjectNode<CO>>>,
}

impl<'a, CO: CoordinateType> Iterator for ObjectNodeIter<'a, CO>{
    type Item = &'a ObjectNode<CO>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.prior{
            self.stack.push_back(root.children.iter());
            self.prior = None;
            return Some(root);
        }
        while let Some(mut iter) = self.stack.back_mut(){
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

impl<C:CoordinateType> ObjectNode<C>{
    pub fn position(&self) -> Vec3{
        return self.position.clone();
    }

    pub fn direction(&self) -> Quaternion{
        return self.direction.clone();
    }

    pub fn add_child(&mut self, child: ObjectNode<C>){
        self.children.insert(child.tag.clone(), child);
    }

    pub fn remove_child(&mut self, tag: String){
        self.children.remove(&tag);
    }

    pub fn child(&self, tag: &str) -> Option<&ObjectNode<C>>{
        return self.children.get(&tag.to_string());
    }

    pub fn child_mut(&mut self, tag: &str) -> Option<&mut ObjectNode<C>>{
        return self.children.get_mut(&tag.to_string());
    }

    pub fn iter(&self) -> ObjectNodeIter<C>{
        let mut stack = VecDeque::new();
        return ObjectNodeIter{
            prior: Some(self),
            stack: stack
        };
    }
}

impl ObjectNode<Local>{
    pub fn new(tag: &str) -> Self{
        return ObjectNode::from(tag,vec![]);
    }
    pub fn from(tag: &str, polygons:Vec<Polygon<Local>>) -> Self {
        return ObjectNode{
            tag: tag.to_string(),
            attribute: Rc::new(RefCell::new(PresetObjectNodeAttribute::Normal())),
            position: Vec3::default(),
            direction: Quaternion::default(),
            polygons: polygons,
            c_particles: vec![],
            children: Default::default()
        };
    }
    pub fn generate_global_nodes(&self) -> ObjectNode<Global>{
        let mut iter = self.iter();
        let mut stack_global:VecDeque<ObjectNode<Global>> = VecDeque::new();
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
                    material: p.material,
                    _ph: Default::default(),
                });
            }
            for p in &now.c_particles {
                child.c_particles.push(CParticle {
                    position: child.position + child.direction.rotate(&p.position),
                    velocity: p.velocity,
                    color: p.color,
                    c: p.c,
                    threshold: p.threshold,
                    mode: p.mode,
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
    use crate::ascia::core::{Local, ObjectNode, Polygon};
    use crate::ascia::core::PresetObjectNodeAttribute::Normal;
    use crate::ascia::math::{Matrix33, Quaternion, Vec3};

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
        let mut a: ObjectNode<Local> = ObjectNode{
            tag: "a".to_string(),
            attribute: Rc::new(RefCell::new(Normal())),
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
        let b: ObjectNode<Local> = ObjectNode{
            tag: "b".to_string(),
            attribute: Rc::new(RefCell::new(Normal())),
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
        let mut c: ObjectNode<Local> = ObjectNode{
            tag: "c".to_string(),
            attribute: Rc::new(RefCell::new(Normal())),
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
        let d: ObjectNode<Local> = ObjectNode{
            tag: "d".to_string(),
            attribute: Rc::new(RefCell::new(Normal())),
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
                let color:Color8bit = rc.color.into();
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

pub struct AsciaEngine{
    pub genesis_local: ObjectNode<Local>,
    pub genesis_global: ObjectNode<Global>,
    pub viewport: RefCell<Box<dyn Viewport>>,
    pub wgpu_daq: Option<(wgpu::Device,wgpu::Queue)>,
    engine_time:Duration,
    engine_started:Instant
}

impl AsciaEngine{
    pub fn new(width:usize,height:usize) -> Self{
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let mut limits = wgpu::Limits::default();
        let mut wgpu_daq:Option<(wgpu::Device,wgpu::Queue)> = None;
        limits.max_bind_groups = 8;
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
            genesis_local: ObjectNode::new("genesis"),
            genesis_global: ObjectNode{
                tag: "genesis".to_string(),
                attribute: Rc::new(RefCell::new(PresetObjectNodeAttribute::Normal())),
                position: Default::default(),
                direction: Default::default(),
                polygons: vec![],
                c_particles: vec![],
                children: Default::default(),
            },
            viewport: RefCell::new(Box::new(ViewportStdout::new(width, height))),
            wgpu_daq: wgpu_daq,
            engine_time:Duration::ZERO,
            engine_started:Instant::now()
        }
    }

    pub fn render(&self, camera_node: &ObjectNode<Global>){
        if let PresetObjectNodeAttribute::Camera(camera) = &*camera_node.attribute.borrow() {
            let result = camera.render(camera_node, self);
            self.viewport.borrow_mut().display(&result);
        }
    }

    pub fn engine_time(&self) -> Duration{
        return self.engine_time.clone();
    }

    pub fn sync_engine_time(&mut self){
        self.engine_time = self.engine_started.elapsed();
    }
    
    pub fn update_global_nodes(&mut self){
        self.genesis_global = self.genesis_local.generate_global_nodes();
    }
}