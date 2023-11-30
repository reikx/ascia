struct Material{
    color: vec3<f32>,
    mode: u32,
    priority: u32
}

struct Polygon{
    vertices: mat3x3<f32>,
    material: Material,
}

struct CParticle{
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec3<f32>,
    c: u32,
    threshold:f32,
    mode:u32
}

struct Light{
    position: vec3<f32>,
    color: vec3<f32>,
    power: f32
}

struct Ray{
    position: vec3<f32>,
    direction: vec3<f32>,
}

struct Intersection{
    ray: Ray,
    polygon_index: u32,
    depth: f32,
    intersection_position_on_polygon: vec2<f32>,
    intersection_position_global: vec3<f32>
}

struct CPIntersection{
    ray: Ray,
    particle_index: u32,
    depth: f32,
    distance: f32
}

struct Dot{
    color:u32,
    depth:f32,
    priority:u32
}

struct CPResult{
    depth: atomic<u32>,
    particle_index: atomic<u32>,
    semaphore_flag: atomic<u32>
}

struct RaytracingSetting{
    camera_position:vec3<f32>,
    camera_direction:vec4<f32>,
    screen_width:u32,
    screen_height:u32,
    angle_of_view:vec2<f32>,
    max_reflection:u32,
    sampling_size:u32,
    sampling_threshold:u32
}

@group(0) @binding(0) var<storage,read> settings: RaytracingSetting;
@group(0) @binding(1) var<storage,read> polygons: array<Polygon>;
@group(0) @binding(2) var<storage,read> lights: array<Light>;
@group(0) @binding(3) var<storage,read> char_mapper: array<u32>;
@group(0) @binding(4) var<storage,read_write> bvh_tree: array<mat2x3<f32>>;
@group(0) @binding(5) var<storage,read_write> bvh_flag: array<u32>;
@group(0) @binding(6) var<storage,read> particles: array<CParticle>;
@group(0) @binding(7) var<storage,read_write> cp_counter: array<atomic<u32>>;

@group(1) @binding(0) var<storage,read_write> dots: array<Dot>;
@group(1) @binding(1) var<storage,read_write> chars: array<u32>;
@group(1) @binding(2) var<storage,read_write> cp_results: array<CPResult>;

fn rotate_vec(q : vec4<f32>, v: vec3<f32>) -> vec3<f32>{
    let d1:f32 = q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    let d2:f32 = q[1] * v[0] + q[2] * v[1] + q[3] * v[2];
    return vec3<f32>(
        2.0 * d2 * q[1] + (q[0] * q[0] - d1) * v[0] + 2.0 * q[0] * (q[2] * v[2] - q[3] * v[1]),
        2.0 * d2 * q[2] + (q[0] * q[0] - d1) * v[1] + 2.0 * q[0] * (q[3] * v[0] - q[1] * v[2]),
        2.0 * d2 * q[3] + (q[0] * q[0] - d1) * v[2] + 2.0 * q[0] * (q[1] * v[1] - q[2] * v[0])
    );
}

fn generate_ray_1x(global_id:vec3<u32>) -> Ray{
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
         var base = vec3<f32>(
             1.0,
             tan(settings.angle_of_view[1] * 0.5) * (1.0 - 2.0 * (f32(global_id.y) / f32(settings.screen_height))),
             tan(settings.angle_of_view[0] * 0.5) * (1.0 - 2.0 * (f32(global_id.x) / f32(settings.screen_width)))
         );
         let direction = rotate_vec(settings.camera_direction,base);
         return Ray(settings.camera_position, direction);
    }
    return Ray(vec3<f32>(0.0,0.0,0.0),vec3<f32>(0.0,0.0,0.0));
}

fn generate_ray_3x(global_id:vec3<u32>) -> Ray{
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
         var base = vec3<f32>(
             1.0,
             tan(settings.angle_of_view[1] * 0.5) * (1.0 - 2.0 * f32(global_id.y * 3u + select(0u,select(1u,2u,global_id.z == 5u || global_id.z == 6u || global_id.z == 7u),global_id.z != 0u && global_id.z != 1u && global_id.z != 2u)) / f32(settings.screen_height * 3u)),
             tan(settings.angle_of_view[0] * 0.5) * (1.0 - 2.0 * f32(global_id.x * 3u + select(0u,select(1u,2u,global_id.z == 2u || global_id.z == 4u || global_id.z == 7u),global_id.z != 0u && global_id.z != 3u && global_id.z != 5u)) / f32(settings.screen_width * 3u))
         );
         let direction = rotate_vec(settings.camera_direction,base);
         return Ray(settings.camera_position, direction);
    }
    return Ray(vec3<f32>(0.0,0.0,0.0),vec3<f32>(0.0,0.0,0.0));
}

@compute
@workgroup_size(64,1,1)
fn init_bvh(@builtin(global_invocation_id) global_id:vec3<u32>,@builtin(num_workgroups) nums:vec3<u32>){
    let polygon_index = global_id.x;
    let n = arrayLength(&polygons);
    let tree_width = 1u << (firstLeadingBit(n - 1u) + 1u);
    var current_width = tree_width;
    if(global_id.x >= tree_width){
        return;
    }

    storageBarrier();


    var node_index = current_width + global_id.x;
    loop{
        if(global_id.x >= current_width){
            return;
        }
        let flag = bvh_flag[(node_index >> 1u)];
        if(flag != 0u){
            current_width >>= 6u;
            node_index = current_width + global_id.x;
        }
        else{
            break;
        }
    }

    if(nums.x == 1u && false){
        bvh_tree[global_id.x][0][0] = f32(global_id.x);
        bvh_tree[global_id.x][0][1] = f32(node_index);
        bvh_tree[global_id.x][0][2] = f32(bvh_flag[(node_index >> 1u)]);
        //atomicStore(&bvh_flag[global_id.x],1u);
        return;
    }

    if(node_index < current_width || node_index >= (current_width << 1u)){
        return;
    }

    if(node_index >= tree_width){
        bvh_tree[node_index][0] = min(polygons[polygon_index].vertices[0],min(polygons[polygon_index].vertices[1],polygons[polygon_index].vertices[2]));
        bvh_tree[node_index][1] = max(polygons[polygon_index].vertices[0],max(polygons[polygon_index].vertices[1],polygons[polygon_index].vertices[2]));
        bvh_flag[node_index] = 1u;
    }
    //bvh_tree[tree_index][0] = vec3<f32>(255.0,f32(tree_index),f32(polygon_index));
    //bvh_tree[tree_index][1] = vec3<f32>(135135.0,f32(n),135135.0);

    var i = 0u;
    loop{
        if(i == 6u){
            return;
        }
        if (node_index == 1u) {
            return;
        }
        let parent_index = node_index >> 1u;
        if ((node_index & 1u) == 0u){
            storageBarrier();
            let flag1:bool = any(bvh_tree[node_index][0] != vec3<f32>() || bvh_tree[node_index][1] != vec3<f32>());
            let flag2:bool = any(bvh_tree[(node_index ^ 1u)][0] != vec3<f32>() || bvh_tree[(node_index ^ 1u)][1] != vec3<f32>());

            if (flag1){
                if(flag2){
                    bvh_tree[parent_index][0] = min(bvh_tree[node_index][0],bvh_tree[node_index ^ 1u][0]);
                    bvh_tree[parent_index][1] = max(bvh_tree[node_index][1],bvh_tree[node_index ^ 1u][1]);
                    bvh_flag[parent_index] |= 16u;
                }
                else{
                    bvh_tree[parent_index] = bvh_tree[node_index];
                    bvh_flag[parent_index] |= 32u;
                }
            }
            else{
                if(flag2){
                    bvh_tree[parent_index] = bvh_tree[node_index ^ 1u];
                    bvh_flag[parent_index] |= 64u;
                }
                else{
                    bvh_flag[parent_index] |= 128u;
                }
            }

            node_index = parent_index;
        }
        else{
            return;
        }
        i += 1u;
    }
}

fn project_bvh(ray:Ray,exclude_polygon_index:u32) -> Intersection{
    let p_len = arrayLength(&polygons);
    var nearest_index = 0u;
    var nearest_depth = 10000000.0;

    var i = 1u;
    let tree_width = 1u << (firstLeadingBit(p_len - 1u) + 1u);
    var count = 0u;

    loop{
        count += 1u;
        if(i >= (tree_width << 1u) || count >= 1048576u){
            break;
        }
        let v0 = (bvh_tree[i][0] - ray.position) / ray.direction;
        let v1 = (bvh_tree[i][1] - ray.position) / ray.direction;
        let v_min = min(v0,v1);
        let v_max = max(v0,v1);

        if(any(v_min != v_max) && max(v_min[0],max(v_min[1],v_min[2])) <= min(v_max[0],min(v_max[1],v_max[2]))){
            if(tree_width <= i){
                let polygon_index = i - tree_width;
                if (polygon_index < p_len){
                    let depth = project_f32_faster(ray, polygon_index);
                    if (depth > 0.0 && depth < nearest_depth && polygon_index != exclude_polygon_index){
                        nearest_depth = depth;
                        nearest_index = polygon_index;
                    }
                }
                if ((i & 1u) == 0u){
                    i |= 1u;
                }
                else{
                    i = (i + 1u) >> firstTrailingBit(i + 1u);
                    if(i <= 1u){
                        break;
                    }
                }
            }
            else{
                i <<= 1u;
            }
        }
        else{
            if((i & 1u) == 0u){
                i |= 1u;
            }
            else{
                i = (i + 1u) >> firstTrailingBit(i + 1u);
                if(i <= 1u){
                    break;
                }
            }
        }
    }
    if (nearest_depth < 10000000.0){
        return project(ray, nearest_index);
    }
    else{
        return Intersection();
    }
}

@compute
@workgroup_size(8,8,1)
fn render_1x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < (settings.screen_width) && global_id.y < (settings.screen_height)){
        let ray = generate_ray_1x(global_id);
        let intersection = project_bvh(ray,0xffffffffu);
        if (intersection.depth > 0.0){
            dots[global_id.y * settings.screen_width + global_id.x] = shade(intersection.ray, intersection.polygon_index, intersection.depth, intersection.intersection_position_on_polygon, intersection.intersection_position_global);
        }
        dots_to_chars(global_id);
        cpr_to_chars(global_id);
    }
}

@compute
@workgroup_size(4,2,8)
fn render_3x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height && global_id.z < 8u){
        let ray = generate_ray_3x(global_id);
        let intersection = project_bvh(ray,0xffffffffu);
        if (intersection.depth > 0.0){
            dots[(global_id.y * settings.screen_width + global_id.x) * 8u + global_id.z] = shade(intersection.ray, intersection.polygon_index, intersection.depth, intersection.intersection_position_on_polygon, intersection.intersection_position_global);
        }
        dots_to_chars(global_id);
        cpr_to_chars(global_id);
    }
}

fn project_f32_faster(ray:Ray, polygon_index:u32) -> f32{
    let m = mat3x3<f32>(
        polygons[polygon_index].vertices[0] - ray.position,
        polygons[polygon_index].vertices[1] - ray.position,
        polygons[polygon_index].vertices[2] - ray.position,
    );
    let d = determinant(m);
    let v1 = transpose(mat3x3<f32>(cross(m[1],m[2]) / d, cross(m[2],m[0]) / d, cross(m[0],m[1]) / d)) * ray.direction;
    let psy = 1.0 / (v1[0] + v1[1] + v1[2]);
    if(psy > 0.0){
        let v2 = v1 * psy;
        if (all(vec3<f32>() <= v2) && all(v2 <= vec3<f32>(1.0,1.0,1.0))){
            return max(0.0,length(ray.direction) * psy);
        }
    }
    return 0.0;
}

fn project_cp(ray:Ray, particle_index:u32) -> CPIntersection{
    let particle = particles[particle_index];
    let a = particle.position - ray.position;
    let k = dot(ray.direction,a) / dot(ray.direction,ray.direction);
    let d = length(a - k * ray.direction);
    if(particle.mode == 0u){ // sphere mode
        if (d < particle.threshold){
            return CPIntersection(ray, particle_index, k * length(ray.direction),d);
        }
        else{
            return CPIntersection();
        }
    }
    else if(particle.mode == 1u){ // arg mode
        if(sqrt(dot(a,a) / dot(ray.direction,ray.direction)) * cos(particle.threshold) <= k){
            return CPIntersection(ray, particle_index, k * length(ray.direction),d);
        }
        else{
            return CPIntersection();
        }
    }
    return CPIntersection();
}

fn project(ray:Ray, polygon_index:u32) -> Intersection{
    let m = mat3x3<f32>(
        polygons[polygon_index].vertices[0] - ray.position,
        polygons[polygon_index].vertices[1] - ray.position,
        polygons[polygon_index].vertices[2] - ray.position,
    );
    let d = determinant(m);
    if (d != 0.0) {
        let inv = mat3x3<f32>(
            vec3<f32>(
                (m[1][1] * m[2][2] - m[1][2] * m[2][1]) / d,
                -(m[0][1] * m[2][2] - m[0][2] * m[2][1]) / d,
                (m[0][1] * m[1][2] - m[0][2] * m[1][1]) / d
            ),
            vec3<f32>(
                -(m[1][0] * m[2][2] - m[1][2] * m[2][0]) / d,
                (m[0][0] * m[2][2] - m[0][2] * m[2][0]) / d,
                -(m[0][0] * m[1][2] - m[0][2] * m[1][0]) / d
            ),
            vec3<f32>(
                (m[1][0] * m[2][1] - m[1][1] * m[2][0]) / d,
                -(m[0][0] * m[2][1] - m[0][1] * m[2][0]) / d,
                (m[0][0] * m[1][1] - m[0][1] * m[1][0]) / d
            )
        );

        /*
        let inv = transpose(mat3x3<f32>(
                                        cross(m[1],m[2]) / d,
                                        cross(m[2],m[0]) / d,
                                        cross(m[0],m[1]) / d,
                                    )); */
        let v1 = (inv * ray.direction);
        let psy = 1.0 / (v1[0] + v1[1] + v1[2]);
        if(psy == psy && psy > 0.0){
            let v2 = v1 * psy;
            if ((0.0 <= v2[0] && v2[0] <= 1.0) && (0.0 <= v2[1] && v2[1] <= 1.0) && (0.0 <= v2[2] && v2[2] <= 1.0)){
                let depth = length(ray.direction) * psy;
                if (depth > 0.0){
                    return Intersection(ray, polygon_index, depth, vec2<f32>(v2[1],v2[2]), polygons[polygon_index].vertices[0] + v2[1] * (polygons[polygon_index].vertices[1] - polygons[polygon_index].vertices[0]) + v2[2] * (polygons[polygon_index].vertices[2] - polygons[polygon_index].vertices[0]));
                }
            }
        }
    }
    return Intersection();
}

fn shade(ray:Ray, polygon_index:u32, depth: f32, intersection_position_on_polygon: vec2<f32>, intersection_position_global: vec3<f32>) -> Dot{
    if(polygons[polygon_index].material.mode == 0u) {
        // flat
        return Dot((u32(clamp(polygons[polygon_index].material.color[0],0.0,1.0) * 255.0) << 16u) | (u32(clamp(polygons[polygon_index].material.color[1],0.0,1.0) * 255.0)  << 8u) | u32(clamp(polygons[polygon_index].material.color[2],0.0,1.0) * 255.0) ,depth, polygons[polygon_index].material.priority);
    }
    else if(polygons[polygon_index].material.mode == 1u){
        // lambert
        var result = vec3<f32>(0.0,0.0,0.0);

        let n = arrayLength(&lights);
        for(var i = 0u;i < n;i += 1u){
            let light = lights[i];
            let v1 = polygons[polygon_index].vertices[1] - polygons[polygon_index].vertices[0];
            let v2 = polygons[polygon_index].vertices[2] - polygons[polygon_index].vertices[0];
            let normal = cross(v1,v2);
            let co:f32 = -dot(normalize(intersection_position_global - light.position), normalize(normal));
            if (((co < 0.0) && (dot(normal, (settings.camera_position - intersection_position_global)) < 0.0)) || ((co > 0.0) && (dot(normal, (settings.camera_position - intersection_position_global)) > 0.0))){
                result += (polygons[polygon_index].material.color * light.color) * abs(co) * light.power;
            }
        }
        return Dot((u32(clamp(result[0],0.0,1.0) * 255.0) << 16u) | (u32(clamp(result[1],0.0,1.0) * 255.0)  << 8u) | u32(clamp(result[2],0.0,1.0) * 255.0), depth, polygons[polygon_index].material.priority);
    }
    else if(polygons[polygon_index].material.mode == 2u){
        // lambert with shadow
        var result = vec3<f32>(0.0,0.0,0.0);

        let n = arrayLength(&lights);
        for(var i = 0u;i < n;i += 1u){
            let light = lights[i];
            let v1 = polygons[polygon_index].vertices[1] - polygons[polygon_index].vertices[0];
            let v2 = polygons[polygon_index].vertices[2] - polygons[polygon_index].vertices[0];
            let normal = cross(v1,v2);
            let co:f32 = -dot(normalize(intersection_position_global - light.position), normalize(normal));

            if (project_bvh(Ray(intersection_position_global,light.position - intersection_position_global),polygon_index).depth > 0.0) {
                continue;
            }
            if (((co < 0.0) && (dot(normal, (settings.camera_position - intersection_position_global)) < 0.0)) || ((co > 0.0) && (dot(normal, (settings.camera_position - intersection_position_global)) > 0.0))){
                result += (polygons[polygon_index].material.color * light.color) * abs(co) * light.power;
            }
        }
        return Dot((u32(clamp(result[0],0.0,1.0) * 255.0) << 16u) | (u32(clamp(result[1],0.0,1.0) * 255.0)  << 8u) | u32(clamp(result[2],0.0,1.0) * 255.0), depth, polygons[polygon_index].material.priority);
    }
    return Dot(0u, depth, polygons[polygon_index].material.priority);
}

@compute
@workgroup_size(8,8,1)
fn render_cp(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < (settings.screen_width) && global_id.y < (settings.screen_height)){
        let n = arrayLength(&particles);
        let x = global_id.x;
        let y = global_id.y;

        var nearest_index = 0u;
        var nearest_depth = 10000000.0;

        let ray = generate_ray_1x(global_id);
        var i = 0u;

        loop{
            if (i >= arrayLength(&particles)){
                break;
            }
            let intersection = project_cp(ray, i);
            if (intersection.depth > 0.0 && intersection.depth < nearest_depth){
                nearest_index = i;
                nearest_depth = intersection.depth;
            }
            i += 1u;
        }

        if (nearest_depth < 10000000.0){
            let intersection = project_cp(ray, nearest_index);
            let particle = particles[nearest_index];
            loop{
                if(atomicOr(&cp_results[global_id.y * settings.screen_width + global_id.x].semaphore_flag,1u) == 0u){
                    if (atomicAdd(&cp_counter[nearest_index],1u) == 0u){
                        if(bitcast<u32>(intersection.depth) < atomicMin(&cp_results[global_id.y * settings.screen_width + global_id.x].depth,bitcast<u32>(intersection.depth))){
                            let old = atomicExchange(&cp_results[global_id.y * settings.screen_width + global_id.x].particle_index,nearest_index);
                            atomicStore(&cp_counter[old],0u);
                        }
                        else if(atomicLoad(&cp_results[global_id.y * settings.screen_width + global_id.x].depth) == 0u){
                            atomicStore(&cp_results[global_id.y * settings.screen_width + global_id.x].depth,bitcast<u32>(intersection.depth));
                            atomicStore(&cp_results[global_id.y * settings.screen_width + global_id.x].particle_index,nearest_index);
                        }
                    }
                    atomicStore(&cp_results[global_id.y * settings.screen_width + global_id.x].semaphore_flag,0u);
                    break;
                }
            }
        }
    }
}

fn dots_to_chars(global_id:vec3<u32>){
   if(settings.sampling_size == 1u){
       chars[global_id.y * settings.screen_width + global_id.x] = select(0x20u,(dots[global_id.y * settings.screen_width + global_id.x].color) << 8u | 0x23u,dots[global_id.y * settings.screen_width + global_id.x].depth > 0.0);
   }
   else if(settings.sampling_size == 3u){
       if(global_id.x < settings.screen_width && global_id.y < settings.screen_height && global_id.z == 0u){
           storageBarrier();
           var max_priority = 0u;
           for(var k = 0u;k < 8u;k += 1u){
               max_priority = max(max_priority,dots[(global_id.y * settings.screen_width + global_id.x) * 8u + k].priority);
           }
           var ch = 0u;
           var count = 0u;
           var result_r = 0u;
           var result_g = 0u;
           var result_b = 0u;
           for(var k = 0u;k < 8u;k += 1u){
               ch <<= 1u;
               let d = dots[(global_id.y * settings.screen_width + global_id.x) * 8u + k];
               let intensity = max((d.color >> 16u) & 0xffu,max((d.color >> 8u) & 0xffu,(d.color) & 0xffu));
               if(d.depth > 0.0 && intensity >= settings.sampling_threshold && d.priority == max_priority){
                   ch |= 1u;
                   count += 1u;
                   result_r += (d.color >> 16u) & 0xffu;
                   result_g += (d.color >> 8u) & 0xffu;
                   result_b += (d.color) & 0xffu;
               }
           }
           result_r = min(result_r / count,255u);
           result_g = min(result_g / count,255u);
           result_b = min(result_b / count,255u);
           let color = (result_r << 16u) | (result_g << 8u) | result_b;
           chars[global_id.y * settings.screen_width + global_id.x] = (color << 8u) | char_mapper[ch];
       }
   }
   else{
       chars[global_id.y * settings.screen_width + global_id.x] = 0xff00ff00u | 0x3du;
   }
}

fn cpr_to_chars(global_id:vec3<u32>){
    let index = atomicLoad(&cp_results[global_id.y * settings.screen_width + global_id.x].particle_index);
    let particle = particles[index];
    let depth = bitcast<f32>(atomicLoad(&cp_results[global_id.y * settings.screen_width + global_id.x].depth));
    if (settings.sampling_size == 1u){
        if(depth > 0.0 && (depth < dots[global_id.y * settings.screen_width + global_id.x].depth || dots[global_id.y * settings.screen_width + global_id.x].depth == 0.0)){
            var color = (u32(clamp(particle.color[0],0.0,1.0) * 255.0) << 16u) | (u32(clamp(particle.color[1],0.0,1.0) * 255.0)  << 8u) | u32(clamp(particle.color[2],0.0,1.0) * 255.0);
            chars[global_id.y * settings.screen_width + global_id.x] = (color << 8u) | particle.c;
        }
    }
    else if(settings.sampling_size == 3u){
        if(global_id.x < settings.screen_width && global_id.y < settings.screen_height && global_id.z == 0u){
            if(depth > 0.0 && (depth < dots[(global_id.y * settings.screen_width + global_id.x) * 8u + 4u].depth || dots[(global_id.y * settings.screen_width + global_id.x) * 8u + 4u].depth == 0.0)){
                var color = (u32(clamp(particle.color[0],0.0,1.0) * 255.0) << 16u) | (u32(clamp(particle.color[1],0.0,1.0) * 255.0)  << 8u) | u32(clamp(particle.color[2],0.0,1.0) * 255.0);
                chars[global_id.y * settings.screen_width + global_id.x] = (color << 8u) | particle.c;
            }
        }
    }
}
