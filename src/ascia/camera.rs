use std::alloc::Layout;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::ascia::charmapper;
use crate::ascia::core::{AsciaEngine, Camera, CParticle, CParticleMode, CParticleRayIntersection, FlatMaterial, Global, LambertMaterial, LambertWithShadowMaterial, Light, Material, ObjectNode, Polygon, PolygonRayIntersection, Ray, RaytracingTarget, RayIntersection, RenderChar, PresetPolygonMaterial, PresetCamera, ObjectNodeAttribute, AsciaEnvironment, ObjectNodeAttributeDispatcher, LightDispatcher, MaterialDispatcher, PresetObjectNodeAttributeDispatcher, CameraDispatcher, PresetLight, PresetCParticleMaterial};
use crate::ascia::math::{AABB3D, Vec3};
use crate::ascia::color::{ColorRGBf32, ColorRGBu8};
use crate::ascia::lights::PointLight;

impl<E: AsciaEnvironment, CA:Camera<E>> Material<E, CA, Polygon<E, Global>> for &FlatMaterial{
    type Output = (ColorRGBf32, u32);

    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        (self.color, self.priority)
    }
}

impl<E: AsciaEnvironment, CA:Camera<E>> Material<E, CA, Polygon<E, Global>> for &LambertMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        let mut result = ColorRGBf32::default();
        for node in engine.genesis_global.iter(){
            if let Some(a) = &*node.attribute.borrow(){
                if let Some(light) = a.light(){
                    let co = (node.position - intersection.position).normalize() * intersection.normal;
                    let color = light.ray(node, &intersection.position);
                    if co * (intersection.normal * (camera_node.position - intersection.position)) > 0.0{
                        result += ColorRGBf32{
                            r: ((self.color.r * color.r) * f32::abs(co)) as f32,
                            g: ((self.color.g * color.g) * f32::abs(co)) as f32,
                            b: ((self.color.b * color.b) * f32::abs(co)) as f32,
                        }
                    }
                }
            }
        }
        return (result, self.priority);
    }
}

impl<'a, E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> Material<E, SimpleCamera<E>, Polygon<E, Global>> for &LambertWithShadowMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &SimpleCamera<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        let mut result = ColorRGBf32::default();
        for node in engine.genesis_global.iter(){
            if let Some(a) = &*node.attribute.borrow(){
                if let Some(light) = a.light(){
                    let co = (node.position - intersection.position).normalize() * intersection.normal;
                    let color = light.ray(node, &intersection.position);
                    let mut is_prevented = false;
                    for iter in global_polygons{
                        if let Some(i) = (Ray{
                            position: intersection.position,
                            direction: node.position - intersection.position
                        }).project(iter, &(|i: &PolygonRayIntersection<E, Global>| -> bool { std::ptr::eq(i.polygon, intersection.polygon)})){
                            if i.depth > 0.0{
                                is_prevented = true;
                                break;
                            }
                        }
                    }
                    if is_prevented{
                        continue;
                    }
                    if co * (intersection.normal * (camera_node.position - intersection.position)) > 0.0{
                        result += ColorRGBf32{
                            r: ((self.color.r * color.r) * f32::abs(co)) as f32,
                            g: ((self.color.g * color.g) * f32::abs(co)) as f32,
                            b: ((self.color.b * color.b) * f32::abs(co)) as f32,
                        }
                    }
                }
            }
        }
        return (result, self.priority);
    }
}

impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> Material<E, SimpleBVHCamera<E>, Polygon<E, Global>> for &LambertWithShadowMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &SimpleBVHCamera<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        let mut result = ColorRGBf32{
            r:0.0,
            g:0.0,
            b:0.0
        };
        for node in engine.genesis_global.iter(){
            if let Some(a) = &*node.attribute.borrow() {
                if let Some(light) = a.light() {
                    let co = (node.position - intersection.position).normalize() * intersection.normal;
                    let color = light.ray(node, &intersection.position);
                    let mut is_prevented = Ray{
                        position: intersection.position,
                        direction: node.position - intersection.position
                    }.project(&*camera.polygons_bvh_tree.borrow(), &(|i: &PolygonRayIntersection<E, Global>| -> bool { std::ptr::eq(i.polygon, intersection.polygon)})).is_some();
                    if is_prevented{
                        continue;
                    }
                    if co * (intersection.normal * (camera_node.position - intersection.position)) > 0.0{
                        result += ColorRGBf32{
                            r: ((self.color.r * color.r) * f32::abs(co)) as f32,
                            g: ((self.color.g * color.g) * f32::abs(co)) as f32,
                            b: ((self.color.b * color.b) * f32::abs(co)) as f32,
                        }
                    }
                }
            }
        }
        return (result, self.priority);
    }
}

// auto deref specialization
impl<'a, E: AsciaEnvironment, CA: Camera<E>> Material<E, CA, Polygon<E, Global>> for LambertWithShadowMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output {
        unimplemented!()
    }
}

impl<E: AsciaEnvironment, CA:Camera<E>> Material<E, CA, CParticle<E, Global>> for &FlatMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &CParticleRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        return (self.color, self.priority);
    }
}

impl<E: AsciaEnvironment, CA:Camera<E>> Material<E, CA, CParticle<E, Global>> for &LambertMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &CParticleRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        todo!()
    }
}

impl<E: AsciaEnvironment, CA:Camera<E>> Material<E, CA, CParticle<E, Global>> for &LambertWithShadowMaterial{
    type Output = (ColorRGBf32, u32);
    fn calc_color(&self, intersection: &CParticleRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera: &CA, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> Self::Output{
        todo!()
    }
}



pub struct SimpleCamera<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial>>{
    pub angle_of_view: (f32, f32),
    pub sampling_size: u32,
    pub ph: PhantomData<E>
}

impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial>> Default for SimpleCamera<E>{
    fn default() -> Self {
        return SimpleCamera{
            angle_of_view: (PI / 3.0,PI / 4.0),
            sampling_size: 1,
            ph: Default::default(),
        }
    }
}

impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial>> SimpleCamera<E>{
    pub fn new(angle_of_view: (f32,f32), sampling_size:u32) -> SimpleCamera<E>{
        return SimpleCamera{
            angle_of_view: angle_of_view,
            sampling_size: sampling_size,
            ph: Default::default(),
        }
    }
}


impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, Lights = PresetLight>> ObjectNodeAttribute<E> for SimpleCamera<E> {}

impl<E: AsciaEnvironment<ObjectNodeAttributes = PresetObjectNodeAttributeDispatcher<E>, PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Cameras = PresetCamera<E>, Lights = PresetLight>> From<SimpleCamera<E>> for PresetObjectNodeAttributeDispatcher<E> {
    fn from(value: SimpleCamera<E>) -> Self {
        PresetCamera::SimpleCamera(value).make_attribute_enum()
    }
}

impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> MaterialDispatcher<E, Polygon<E, Global>> for SimpleCamera<E>{
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> (ColorRGBf32, u32) {
        match intersection.polygon.material {
            PresetPolygonMaterial::FlatMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetPolygonMaterial::LambertMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetPolygonMaterial::LambertWithShadowMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
        }
    }
}

impl<E: AsciaEnvironment<PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> MaterialDispatcher<E, CParticle<E, Global>> for SimpleCamera<E>{
    fn calc_color(&self, intersection: &CParticleRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) ->  (ColorRGBf32, u32) {
        match intersection.particle.material {
            PresetCParticleMaterial::FlatMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetCParticleMaterial::LambertMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetCParticleMaterial::LambertWithShadowMaterial(m) => {
                <_ as Material<E, SimpleCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
        }
    }
}

impl<E:AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> Camera<E> for SimpleCamera<E>{
    fn render(&self, node: &ObjectNode<E, Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>> {
        let height = engine.viewport.borrow().height();
        let width = engine.viewport.borrow().width();

        let mut output:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];
        let mut polygon_intersections:Vec<Vec<Option<PolygonRayIntersection<E, Global>>>> = vec![vec![None;self.sampling_size as usize * height];self.sampling_size as usize * width];
        let mut c_particle_intersections:Vec<Vec<Option<CParticleRayIntersection<E, Global>>>> = vec![vec![None; self.sampling_size as usize * height]; self.sampling_size as usize * width];

        let mut global_polygons = vec![];
        let mut global_c_particles = vec![];

        for iter in engine.genesis_global.iter(){
            global_polygons.extend(iter.polygons.clone());
            global_c_particles.extend(iter.c_particles.clone());
            global_c_particles.extend(iter.c_particles.clone());
        }

        let mut c_particle_counters = vec![0u32; global_c_particles.len()];

        if self.sampling_size == 1{
            for x in 0..width{
                for y in 0..height{
                    polygon_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&global_polygons, &|i|{false});
                    c_particle_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&global_c_particles, &|i|{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&global_c_particles[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&global_c_particles[0]).size()] > 0
                    });
                    if let Some(i) = &c_particle_intersections[x][y]{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&global_c_particles[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&global_c_particles[0]).size()] += 1;
                    }
                }
            }

            for x in 0..width{
                for y in 0..height{
                    let mut depth = f32::MAX;
                    if let Some(intersection) = &polygon_intersections[x][y]{
                        output[y][x].c = '#';
                        output[y][x].color = <_ as MaterialDispatcher<E, Polygon<E, Global>>>::calc_color(self, intersection, engine, node, &global_polygons).0.into();
                        depth = intersection.depth;
                    }
                    if let Some(intersection) = &c_particle_intersections[x][y]{
                        output[y][x].c = intersection.particle.c;
                        output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &global_polygons).0.into();
                        depth = intersection.depth;
                    }
                }
            }
        }
        else if self.sampling_size == 3{
            for x in 0..width{
                for y in 0..height{
                    for i in 0..3 {
                        for j in 0..3 {
                            if i == 1 && j == 1{
                                continue;
                            }
                            let ray = Ray{
                                position: node.position,
                                direction: node.direction.rotate(&Vec3{
                                    x:1.0,
                                    y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * (y * 3 + i) as f32 / (height * 3) as f32),
                                    z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * (x * 3 + j) as f32 / (width * 3) as f32),
                                }),
                            };
                            let v = &global_polygons;
                            polygon_intersections[x * 3 + j][y * 3 + i] =  ray.project(v, &|i|{false});
                        }
                    }
                    c_particle_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&global_c_particles, &|i|{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&global_c_particles[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&global_c_particles[0]).size()] > 0
                    });
                    if let Some(i) = &c_particle_intersections[x][y]{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&global_c_particles[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&global_c_particles[0]).size()] += 1;
                    }
                }
            }

            for x in 0..width{
                for y in 0..height{
                    let mut material_results:[(ColorRGBf32, u32);8] = [(Default::default(),0);8];
                    let mut k = 0;

                    let mut max_priority = 0;
                    let mut color_sum: ColorRGBf32 = ColorRGBf32{
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };
                    let mut seg:usize = 0;
                    let mut seg_count:u32 = 0;

                    for i in 0..3 {
                        for j in 0..3 {
                            if i == 1 && j == 1 {
                                continue;
                            }
                            seg <<= 1;
                            if let Some(intersection) = &polygon_intersections[x * 3 + j][y * 3 + i] {
                                material_results[k] = <_ as MaterialDispatcher<E, Polygon<E, Global>>>::calc_color(self, intersection, engine, node, &global_polygons);
                                if max_priority < material_results[k].1{
                                    max_priority = material_results[k].1;
                                    seg = 0;
                                    seg_count = 0;
                                    color_sum = ColorRGBf32{
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0
                                    };
                                }
                                color_sum += material_results[k].0;
                                seg |= 1;
                                seg_count += 1;
                            }
                            k += 1;
                        }
                    }

                    output[y][x].c = charmapper::CHARMAP3X3[seg];
                    output[y][x].color = if seg_count == 0 {
                        ColorRGBu8::default()
                    } else {
                        ColorRGBf32{
                            r: color_sum.r / (seg_count as f32),
                            g: color_sum.g / (seg_count as f32),
                            b: color_sum.b / (seg_count as f32),
                        }.into()
                    };

                    if let Some(intersection) = &c_particle_intersections[x][y]{
                        if let Some(first) = &polygon_intersections[x * 3][y * 3]{
                            if intersection.depth < first.depth{
                                output[y][x].c = intersection.particle.c;
                                output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &global_polygons).0.into();
                            }
                        }
                        else{
                            output[y][x].c = intersection.particle.c;
                            output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &global_polygons).0.into();
                        }
                    }
                }
            }
        }
        return output;
    }
}

pub struct NaiveBVH<T>{
    buf: Vec<Option<AABB3D>>,
    data: Vec<T>,
}

impl<E: AsciaEnvironment> NaiveBVH<Polygon<E, Global>>{
    pub fn from_polygons(polygons: Vec<Polygon<E, Global>>) -> Self{
        let mut tree_width = 1usize;
        while tree_width < polygons.len(){
            tree_width <<= 1;
        }
        let mut tree_buf = vec![None; tree_width << 1];
        for i in 0..polygons.len(){
            let p = &polygons[i];
            tree_buf[tree_width + i] = Some(p.aabb());
        }

        let mut current_width = tree_width;
        while current_width > 0 {
            for i in 0..(current_width >> 1){
                let parent_index = (current_width >> 1) + i;
                if let Some(lhs) = tree_buf[current_width + (i << 1)]{
                    if let Some(rhs) = tree_buf[current_width + (i << 1) + 1]{
                        tree_buf[parent_index] = Some(AABB3D::concat(&lhs, &rhs));
                    }
                    else{
                        tree_buf[parent_index] = Some(lhs);
                    }
                }
                else{
                    if let Some(rhs) = tree_buf[current_width + (i << 1) + 1]{
                        tree_buf[parent_index] = Some(rhs);
                    }
                }
            }
            current_width >>= 1;
        }
        return NaiveBVH { buf: tree_buf, data:polygons};
    }
}

impl<E:AsciaEnvironment> NaiveBVH<CParticle<E, Global>>{
    pub fn from_c_particles(c_particles: Vec<CParticle<E, Global>>, camera_pos: &Vec3) -> Self{
        let mut tree_width = 1usize;
        while tree_width < c_particles.len(){
            tree_width <<= 1;
        }
        let mut tree_buf = vec![None; tree_width << 1];
        for i in 0..c_particles.len(){
            let p = &c_particles[i];
            let aabb = match p.mode {
                CParticleMode::SPHERE => {
                    AABB3D::generate_2(&(p.position - Vec3{
                        x: p.threshold,
                        y: p.threshold,
                        z: p.threshold,
                    }), &(p.position + Vec3{
                        x: p.threshold,
                        y: p.threshold,
                        z: p.threshold,
                    }))
                }
                CParticleMode::ARG => {
                    let r = (p.position - *camera_pos).norm() * f32::tan(p.threshold);
                    AABB3D::generate_2(&(p.position - Vec3{
                        x: r,
                        y: r,
                        z: r,
                    }), &(p.position + Vec3{
                        x: r,
                        y: r,
                        z: r,
                    }))
                }
            };
            tree_buf[tree_width + i] = Some(aabb);
        }

        let mut current_width = tree_width;
        while current_width > 0 {
            for i in 0..(current_width >> 1){
                let parent_index = (current_width >> 1) + i;
                if let Some(lhs) = tree_buf[current_width + (i << 1)]{
                    if let Some(rhs) = tree_buf[current_width + (i << 1) + 1]{
                        tree_buf[parent_index] = Some(AABB3D::concat(&lhs, &rhs));
                    }
                    else{
                        tree_buf[parent_index] = Some(lhs);
                    }
                }
                else{
                    if let Some(rhs) = tree_buf[current_width + (i << 1) + 1]{
                        tree_buf[parent_index] = Some(rhs);
                    }
                }
            }
            current_width >>= 1;
        }
        return NaiveBVH { buf: tree_buf, data: c_particles };
    }
}


impl<T> RaytracingTarget<1> for NaiveBVH<T> where T : RaytracingTarget<0> {
    type Intersection<'a> = <T as RaytracingTarget<0>>::Intersection<'a> where T: 'a;
    fn project_by<'a, F:Fn(&Self::Intersection<'a>) -> bool>(&'a self, ray: &Ray, f: &F) -> Option<Self::Intersection<'a>> {
        let mut nearest:Option<Self::Intersection<'a>> = None;
        let mut stack: VecDeque<usize> = VecDeque::new();
        stack.push_back(1);

        while !stack.is_empty() {
            let i = *stack.back().unwrap();
            if let Some(aabb) = &self.buf[i]{
                if let Some(r) = ray.project(aabb, &|i|{false}){
                    if self.buf.len() / 2 <= i{
                        if let Some(r1) = (&self.data[i - (self.buf.len() / 2)]).project_by(&ray, f){
                            if let Some(r2) = &nearest{
                                if r1.depth() < r2.depth(){
                                    nearest = Some(r1);
                                }
                            }
                            else{
                                nearest = Some(r1);
                            }
                        }
                        while let Some(j) = stack.pop_back(){
                            if j % 2 == 0 {
                                stack.push_back(j + 1);
                                break;
                            }
                        }
                    }
                    else{
                        stack.push_back(i << 1);
                    }
                }
                else{
                    while let Some(j) = stack.pop_back(){
                        if j % 2 == 0 {
                            stack.push_back(j + 1);
                            break;
                        }
                    }
                }
            }
            else{
                while let Some(j) = stack.pop_back(){
                    if j % 2 == 0 {
                        stack.push_back(j + 1);
                        break;
                    }
                }
            }
        }
        return nearest;
    }
}

pub struct SimpleBVHCamera<E: AsciaEnvironment>{
    pub angle_of_view: (f32, f32),
    pub sampling_size: u32,
    polygons_bvh_tree: RefCell<NaiveBVH<Polygon<E, Global>>>,
    c_particles_bvh_tree: RefCell<NaiveBVH<CParticle<E, Global>>>,
}

impl<E: AsciaEnvironment> Default for SimpleBVHCamera<E>{
    fn default() -> Self {
        return SimpleBVHCamera{
            angle_of_view: (PI / 3.0,PI / 4.0),
            sampling_size: 1,
            polygons_bvh_tree: RefCell::new(NaiveBVH::from_polygons(vec![])),
            c_particles_bvh_tree: RefCell::new(NaiveBVH::from_c_particles(vec![], &Default::default())),
        }
    }
}

impl<E: AsciaEnvironment> SimpleBVHCamera<E>{
    pub fn new(angle_of_view: (f32,f32), sampling_size:u32) -> SimpleBVHCamera<E>{
        return SimpleBVHCamera{
            angle_of_view: angle_of_view,
            sampling_size: sampling_size,
            polygons_bvh_tree: RefCell::new(NaiveBVH::from_polygons(vec![])),
            c_particles_bvh_tree: RefCell::new(NaiveBVH::from_c_particles(vec![], &Default::default())),
        }
    }
}

impl<E: AsciaEnvironment> ObjectNodeAttribute<E> for SimpleBVHCamera<E> {}

impl<E: AsciaEnvironment<ObjectNodeAttributes = PresetObjectNodeAttributeDispatcher<E>, PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight, Cameras = PresetCamera<E>>> From<SimpleBVHCamera<E>> for PresetObjectNodeAttributeDispatcher<E> {
    fn from(value: SimpleBVHCamera<E>) -> Self {
        PresetCamera::SimpleBVHCamera(value).make_attribute_enum()
    }
}


impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> MaterialDispatcher<E, Polygon<E, Global>> for SimpleBVHCamera<E>{
    fn calc_color(&self, intersection: &PolygonRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) -> (ColorRGBf32, u32) {
        match intersection.polygon.material {
            PresetPolygonMaterial::FlatMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetPolygonMaterial::LambertMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetPolygonMaterial::LambertWithShadowMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, Polygon<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
        }
    }
}

impl<E: AsciaEnvironment<PolygonMaterials = PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> MaterialDispatcher<E, CParticle<E, Global>> for SimpleBVHCamera<E>{
    fn calc_color(&self, intersection: &CParticleRayIntersection<E, Global>, engine: &AsciaEngine<E>, camera_node: &ObjectNode<E, Global>, global_polygons: &Vec<Polygon<E, Global>>) ->  (ColorRGBf32, u32) {
        match intersection.particle.material {
            PresetCParticleMaterial::FlatMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetCParticleMaterial::LambertMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
            PresetCParticleMaterial::LambertWithShadowMaterial(m) => {
                <_ as Material<E, SimpleBVHCamera<E>, CParticle<E, Global>>>::calc_color(&(&m), intersection, engine, self, camera_node, global_polygons)
            }
        }
    }
}
impl<E: AsciaEnvironment<PolygonMaterials=PresetPolygonMaterial, CParticleMaterials=PresetCParticleMaterial, Lights = PresetLight>> Camera<E> for SimpleBVHCamera<E>{
    fn render(&self, node: &ObjectNode<E, Global>, engine: &AsciaEngine<E>) -> Vec<Vec<RenderChar>> {
        let height = engine.viewport.borrow().height();
        let width = engine.viewport.borrow().width();

        let mut output:Vec<Vec<RenderChar>> = vec![vec![RenderChar::default();width];height];
        let mut polygon_intersections:Vec<Vec<Option<PolygonRayIntersection<E, Global>>>> = vec![vec![None;self.sampling_size as usize * height];self.sampling_size as usize * width];
        let mut c_particle_intersections:Vec<Vec<Option<CParticleRayIntersection<E, Global>>>> = vec![vec![None; self.sampling_size as usize * height]; self.sampling_size as usize * width];

        let mut global_polygons = vec![];
        let mut global_c_particles = vec![];
        for iter in engine.genesis_global.iter(){
            global_polygons.extend(iter.polygons.clone());
            global_c_particles.extend(iter.c_particles.clone());
        }

        let mut c_particle_counters = vec![0u32; global_c_particles.len()];

        *self.polygons_bvh_tree.borrow_mut() = NaiveBVH::from_polygons(global_polygons);
        *self.c_particles_bvh_tree.borrow_mut() = NaiveBVH::from_c_particles(global_c_particles, &node.position);
        
        let polygons_bvh_tree = self.polygons_bvh_tree.borrow();
        let c_particles_bvh_tree = self.c_particles_bvh_tree.borrow();

        if self.sampling_size == 1{
            for x in 0..width{
                for y in 0..height{
                    polygon_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&*polygons_bvh_tree, &|i|{false});
                    c_particle_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&*c_particles_bvh_tree, &|i|{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&c_particles_bvh_tree.data[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&c_particles_bvh_tree.data[0]).size()] > 0
                    });
                    if let Some(i) = &c_particle_intersections[x][y]{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&c_particles_bvh_tree.data[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&c_particles_bvh_tree.data[0]).size()] += 1;
                    }
                }
            }

            for x in 0..width{
                for y in 0..height{
                    let mut depth = f32::MAX;
                    if let Some(intersection) = &polygon_intersections[x][y]{
                        output[y][x].c = '#';
                        output[y][x].color = <_ as MaterialDispatcher<E, Polygon<E, Global>>>::calc_color(self, intersection, engine, node, &polygons_bvh_tree.data).0.into();
                        depth = intersection.depth;
                    }
                    if let Some(intersection) = &c_particle_intersections[x][y]{
                        output[y][x].c = intersection.particle.c;
                        output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &polygons_bvh_tree.data).0.into();
                        depth = intersection.depth;
                    }
                }
            }
        }
        else if self.sampling_size == 3{
            for x in 0..width{
                for y in 0..height{
                    for i in 0..3 {
                        for j in 0..3 {
                            if i == 1 && j == 1{
                                continue;
                            }
                            polygon_intersections[x * 3 + j][y * 3 + i] = Ray{
                                position: node.position,
                                direction: node.direction.rotate(&Vec3{
                                    x:1.0,
                                    y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * (y * 3 + i) as f32 / (height * 3) as f32),
                                    z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * (x * 3 + j) as f32 / (width * 3) as f32),
                                }),
                            }.project(&*polygons_bvh_tree, &|i|{false});
                        }
                    }
                    c_particle_intersections[x][y] = Ray{
                        position: node.position,
                        direction: node.direction.rotate(&Vec3{
                            x:1.0,
                            y:f32::tan(self.angle_of_view.1 * 0.5) * (1.0 - 2.0 * y as f32 / height as f32),
                            z:f32::tan(self.angle_of_view.0 * 0.5) * (1.0 - 2.0 * x as f32 / width as f32),
                        })
                    }.project(&*c_particles_bvh_tree, &|i|{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&c_particles_bvh_tree.data[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&c_particles_bvh_tree.data[0]).size()] > 0
                    });
                    if let Some(i) = &c_particle_intersections[x][y]{
                        c_particle_counters[((i.particle as *const CParticle<E, Global>) as usize - (&c_particles_bvh_tree.data[0] as *const CParticle<E, Global>) as usize) / Layout::for_value(&c_particles_bvh_tree.data[0]).size()] += 1;
                    }
                }
            }

            for x in 0..width{
                for y in 0..height{
                    let mut material_results:[(ColorRGBf32, u32);8] = [(Default::default(),0);8];
                    let mut k = 0;
                    
                    let mut max_priority = 0;
                    let mut color_sum: ColorRGBf32 = ColorRGBf32{
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };
                    let mut seg:usize = 0;
                    let mut seg_count:u32 = 0;

                    for i in 0..3 {
                        for j in 0..3 {
                            if i == 1 && j == 1 {
                                continue;
                            }
                            seg <<= 1;
                            if let Some(intersection) = &polygon_intersections[x * 3 + j][y * 3 + i] {
                                material_results[k] = <_ as MaterialDispatcher<E, Polygon<E, Global>>>::calc_color(self, intersection, engine, node, &polygons_bvh_tree.data);
                                if max_priority < material_results[k].1{
                                    max_priority = material_results[k].1;
                                    seg = 0;
                                    seg_count = 0;
                                    color_sum = ColorRGBf32{
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0
                                    };
                                }
                                color_sum += material_results[k].0;
                                seg |= 1;
                                seg_count += 1;
                            }
                            k += 1;
                        }
                    }
                    
                    output[y][x].c = charmapper::CHARMAP3X3[seg];
                    output[y][x].color = if seg_count == 0 {
                        ColorRGBu8::default()
                    } else {
                        ColorRGBf32{
                            r: color_sum.r / (seg_count as f32),
                            g: color_sum.g / (seg_count as f32),
                            b: color_sum.b / (seg_count as f32),
                        }.into()
                    };

                    if let Some(intersection) = &c_particle_intersections[x][y]{
                        if let Some(first) = &polygon_intersections[x * 3][y * 3]{
                            if intersection.depth < first.depth{
                                output[y][x].c = intersection.particle.c;
                                output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &polygons_bvh_tree.data).0.into();
                            }
                        }
                        else{
                            output[y][x].c = intersection.particle.c;
                            output[y][x].color = <_ as MaterialDispatcher<E, CParticle<E, Global>>>::calc_color(self, intersection, engine, node, &polygons_bvh_tree.data).0.into();
                        }
                    }
                }
            }
        }
        return output;
    }
}