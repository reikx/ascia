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

struct Material{
    color: vec3<f32>,
    mode: u32,
    priority: u32
}

struct PointLight{
    position: vec3<f32>,
    color: vec3<f32>,
    power: f32
}

struct Ray{
    position: vec3<f32>,
    direction: vec3<f32>,
}

struct PolygonRayIntersection{
    polygon_id: u32,
    depth: f32,
    ray: Ray,
    position: vec3<f32>,
    position_on_polygon: vec2<f32>,
    normal: vec3<f32>,
}

struct CParticleRayIntersection{
    particle_id: u32,
    depth: f32,
    ray: Ray,
    position: vec3<f32>,
    distance: f32
}

struct RaytracingSetting{
    camera_position:vec3<f32>,
    camera_direction:vec4<f32>,
    screen_width:u32,
    screen_height:u32,
    angle_of_view:vec2<f32>,
    max_reflection:u32,
    sampling_size:u32,
    sampling_threshold:u32,
    render_range_radius: f32
}

@group(0) @binding(0) var<storage,read> settings: RaytracingSetting;
@group(0) @binding(1) var<storage,read> char_mapper: array<u32>;

@group(1) @binding(0) var<storage,read_write> rendered_chars: array<u32>;

@group(2) @binding(0) var<storage,read> polygons: array<Polygon>;
@group(2) @binding(3) var<storage,read_write> intersections_polygons: array<PolygonRayIntersection>;

@group(3) @binding(0) var<storage,read> c_particles: array<CParticle>;
@group(3) @binding(3) var<storage,read_write> intersections_c_particles: array<CParticleRayIntersection>;
@group(3) @binding(4) var<storage,read_write> c_particle_counters: array<atomic<u32>>;

@group(4) @binding(0) var<storage,read> pointlights: array<PointLight>;

/*
 * utils
 */

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


fn convert_rgbf32withpriority_to_rendered_char(data: vec4<f32>, c: u32) -> u32{
    return (u32(clamp(data[0],0.0,1.0) * 255.0) << 24u) | (u32(clamp(data[1],0.0,1.0) * 255.0)  << 16u) | (u32(clamp(data[2],0.0,1.0) * 255.0) << 8u) | c;
}

/*
 * raycaster
 */

fn project_polygon(ray: Ray, polygon_id:u32) -> PolygonRayIntersection{
    let m = mat3x3<f32>(
        polygons[polygon_id].vertices[0] - ray.position,
        polygons[polygon_id].vertices[1] - ray.position,
        polygons[polygon_id].vertices[2] - ray.position,
    );
    let d = determinant(m);
    let v1 = transpose(mat3x3<f32>(cross(m[1],m[2]) / d, cross(m[2],m[0]) / d, cross(m[0],m[1]) / d)) * ray.direction;
    let psy = 1.0 / (v1[0] + v1[1] + v1[2]);
    if(psy > 0.0){
        let v2 = v1 * psy;
        if (all(vec3<f32>() <= v2) && all(v2 <= vec3<f32>(1.0,1.0,1.0))){
            return PolygonRayIntersection(polygon_id, max(0.0,length(ray.direction) * psy), ray, ray.position + ray.direction * psy, vec2<f32>(v2.y, v2.z), normalize(cross(m[1] - m[0], m[2] - m[0])));
        }
    }
    return PolygonRayIntersection(polygon_id, settings.render_range_radius, ray, vec3<f32>(), vec2<f32>(), vec3<f32>());
}

fn project_polygons(ray: Ray, exclude_polygon_id:u32) -> PolygonRayIntersection{
    let polygons_len = arrayLength(&polygons);
    var nearest_intersection = PolygonRayIntersection(2147483649u, settings.render_range_radius, ray, vec3<f32>(), vec2<f32>(), vec3<f32>());

    var polygon_id = 0u;
    loop{
        if (polygon_id >= polygons_len){
            break;
        }
        let result = project_polygon(ray, polygon_id);
        if (result.depth > 0.0 && result.depth < nearest_intersection.depth && polygon_id != exclude_polygon_id){
            nearest_intersection = result;
        }
        polygon_id += 1u;
    }
    return nearest_intersection;
}

fn project_c_particle(ray: Ray, c_particle_id:u32) -> CParticleRayIntersection{
    let c_particle = c_particles[c_particle_id];
    let a = c_particle.position - ray.position;
    let k = dot(ray.direction,a) / dot(ray.direction,ray.direction);
    let d = length(a - k * ray.direction);
    if(c_particle.mode == 0u){ // sphere mode
        if (d < c_particle.threshold){
            return CParticleRayIntersection(c_particle_id, length(k * ray.direction), ray, ray.position + k * ray.direction, d);
        }
    }
    else if(c_particle.mode == 1u){ // arg mode
        if(sqrt(dot(a,a) / dot(ray.direction,ray.direction)) * cos(c_particle.threshold) <= k){
            return CParticleRayIntersection(c_particle_id, length(k * ray.direction), ray, ray.position + k * ray.direction, d);
        }
    }
    return CParticleRayIntersection(c_particle_id, settings.render_range_radius, ray, vec3<f32>(), 0.0);
}

fn project_c_particles(ray: Ray, exclude_c_particle_id: u32) -> CParticleRayIntersection{
    let c_particles_len = arrayLength(&c_particles);
    var nearest_intersection = CParticleRayIntersection(2147483649u, settings.render_range_radius, ray, vec3<f32>(), 0.0);

    var c_particle_id = 1u;
    loop{
        if(c_particle_id >= c_particles_len){
            break;
        }
        let result = project_c_particle(ray, c_particle_id);
        if (result.depth > 0.0 && result.depth < nearest_intersection.depth && c_particle_id != exclude_c_particle_id){
            nearest_intersection = result;
        }
        c_particle_id += 1u;
    }
    return nearest_intersection;
}

/*
 *  main renderer
 */

@compute
@workgroup_size(8,8,1)
fn calc_intersections_polygons_1x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
        let index = global_id.y * settings.screen_width + global_id.x;
        intersections_polygons[index] = project_polygons(generate_ray_1x(global_id), 2147483649u);
    }
}

@compute
@workgroup_size(4,2,8)
fn calc_intersections_polygons_3x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height && global_id.z < 8u){
        let index = (global_id.y * settings.screen_width + global_id.x) * 8u + global_id.z;
        intersections_polygons[index] = project_polygons(generate_ray_3x(global_id), 2147483649u);
    }
}

@compute
@workgroup_size(8,8,1)
fn calc_intersections_c_particles(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
        let index = global_id.y * settings.screen_width + global_id.x;
        let intersection_1 = project_c_particles(generate_ray_1x(global_id), 2147483649u);
        let old_count_1 = atomicAdd(&c_particle_counters[intersection_1.particle_id], 1u);
        if (old_count_1 == 0u){
            intersections_c_particles[index] = intersection_1;
        }
        else {
            let intersection_2 = project_c_particles(generate_ray_1x(global_id), intersection_1.particle_id);
            let old_count_2 = atomicAdd(&c_particle_counters[intersection_2.particle_id], 1u);
            if (old_count_2 == 0u){
                intersections_c_particles[index] = intersection_2;
            }
        }
    }
}

@compute
@workgroup_size(8,8,1)
fn calc_chars_1x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
        let index = global_id.y * settings.screen_width + global_id.x;
        var depth_min = settings.render_range_radius;
        var render_flag = 0u;

        if (intersections_polygons[index].depth < depth_min){
            render_flag = 1u;
            depth_min = intersections_polygons[index].depth;
        }

        if (intersections_c_particles[index].depth < depth_min){
            render_flag = 2u;
        }

        if (render_flag == 1u){
            rendered_chars[index] = convert_rgbf32withpriority_to_rendered_char(calc_color_polygon(index), char_mapper[255u]);
        }
        else if(render_flag == 2u){
            rendered_chars[index] = convert_rgbf32withpriority_to_rendered_char(calc_color_c_particle(index), c_particles[intersections_c_particles[index].particle_id].c);
        }
        else{
            rendered_chars[index] = 0x00ffff00u | 0x20u;
        }
    }
}

@compute
@workgroup_size(8,8,1)
fn calc_chars_3x(@builtin(global_invocation_id) global_id:vec3<u32>){
    if (global_id.x < settings.screen_width && global_id.y < settings.screen_height){
        let index_1x = global_id.y * settings.screen_width + global_id.x;
        var depth_min = settings.render_range_radius;
        var render_flag = 0u;

        if (intersections_polygons[index_1x * 8u].depth < depth_min){
            render_flag = 1u;
            depth_min = intersections_polygons[index_1x * 8u].depth;
        }
        if (intersections_c_particles[index_1x].depth < depth_min){
            render_flag = 2u;
        }

        if (render_flag == 1u){
            var max_priority = 0u;
            var results = array<vec4<f32>, 8u>();
            var color_sum = vec4<f32>();

            var seg = 0u;
            var p_count = 0u;
            var tmp = 0u;
            for (var i = 0u; i < 8u; i += 1u){
                results[i] = calc_color_polygon(index_1x * 8u + i);

                tmp = select(0u, 1u, intersections_polygons[index_1x * 8u + i].depth < settings.render_range_radius);
                if (max_priority < bitcast<u32>(results[i][3])){
                    max_priority = bitcast<u32>(results[i][3]);
                    seg = 0u;
                    p_count = 0u;
                    color_sum = vec4<f32>();
                }
                seg <<= 1u;
                seg |= tmp;
                p_count += tmp;
                color_sum += f32(tmp) * results[i];
            }
            rendered_chars[index_1x] = convert_rgbf32withpriority_to_rendered_char(select(vec4<f32>(), color_sum / f32(p_count), p_count > 0u), char_mapper[seg]);
             /* if (p_count > 0u){
                 rendered_chars[index_1x] = 0x00ffff00u | 0x70u;
             } */
            // rendered_chars[index_1x] = 0x00ffff00u | 0x70u;
           // rendered_chars[index_1x] = convert_rgbf32withpriority_to_rendered_char(vec4<f32>(f32(intersections_polygons[index_1x * 8u].polygon_id), f32(intersections_polygons[index_1x * 8u].polygon_id),f32(intersections_polygons[index_1x * 8u].polygon_id), 0.0), char_mapper[255u]);
        }
        else if(render_flag == 2u){
            rendered_chars[index_1x] = convert_rgbf32withpriority_to_rendered_char(calc_color_c_particle(index_1x), c_particles[intersections_c_particles[index_1x].particle_id].c);
        }
        else{
            rendered_chars[index_1x] = 0x00ffff00u | 0x20u;
        }
        // rendered_chars[index_1x] = 0x00ffff00u | 0x70u;
    }
}

/*
 * materials
 */

fn calc_color_polygon(intersection_id: u32) -> vec4<f32>{
    let intersection = intersections_polygons[intersection_id];
    let polygon = polygons[intersections_polygons[intersection_id].polygon_id];
    if (polygon.material.mode == 0u){
        return calc_color_polygon_flat(intersection_id);
    }
    else if(polygon.material.mode == 1u){
        return calc_color_polygon_lambert(intersection_id);
    }
    else if(polygon.material.mode == 2u){
        return calc_color_polygon_lambert_with_shadow_projection(intersection_id);
    }
    return vec4<f32>(100000000.0, 0.0, 100000000.0, bitcast<f32>(0xffffffffu));
}

fn calc_color_polygon_flat(intersection_id: u32) -> vec4<f32>{
    let material = polygons[intersections_polygons[intersection_id].polygon_id].material;
    return vec4<f32>(material.color[0], material.color[1], material.color[2], bitcast<f32>(material.priority));
}

fn calc_color_polygon_lambert(intersection_id: u32) -> vec4<f32>{
    let intersection = intersections_polygons[intersection_id];
    let polygon = polygons[intersections_polygons[intersection_id].polygon_id];
    var result_color = vec3<f32>(0.0, 0.0, 0.0);
    let n = arrayLength(&pointlights);
    for(var i = 0u; i < n; i += 1u){
        let light = pointlights[i];
        let co:f32 = dot(normalize(light.position - intersection.position), intersection.normal);
        if (co * dot((intersection.ray.position - intersection.position), intersection.normal) > 0.0){
            result_color += (polygon.material.color * light.color) * abs(co) * light.power;
        }
    }
    return vec4<f32>(result_color[0], result_color[1], result_color[2], bitcast<f32>(polygon.material.priority));
}

fn calc_color_polygon_lambert_with_shadow_projection(intersection_id: u32) -> vec4<f32>{
    let intersection = intersections_polygons[intersection_id];
    let polygon = polygons[intersections_polygons[intersection_id].polygon_id];
    var result_color = vec3<f32>(0.0, 0.0, 0.0);
    let n = arrayLength(&pointlights);
    for(var i = 0u; i < n; i += 1u){
        let light = pointlights[i];
        if (project_polygons(Ray(intersection.position, light.position - intersection.position), intersections_polygons[intersection_id].polygon_id).depth < settings.render_range_radius) {
            continue;
        }
        let co:f32 = dot(normalize(light.position - intersection.position), intersection.normal);
        if (co * dot((intersection.ray.position - intersection.position), intersection.normal) > 0.0){
            result_color += (polygon.material.color * light.color) * abs(co) * light.power;
        }
    }
    return vec4<f32>(result_color[0], result_color[1], result_color[2], bitcast<f32>(polygon.material.priority));
}

fn calc_color_c_particle(intersection_id: u32) -> vec4<f32>{
    let c_particle = c_particles[intersections_c_particles[intersection_id].particle_id];
    return vec4<f32>(c_particle.color[0], c_particle.color[1], c_particle.color[2], 0.0);
}