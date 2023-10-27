use clap::Parser;
use rerun::{
    datatypes::Vec3D,
    external::{anyhow, glam, re_log},
    Mesh3D, RecordingStream, RecordingStreamBuilder,
};

#[derive(Debug, clap::Parser)]
#[clap(author, version, about)]
struct Args {
    #[command(flatten)]
    rerun: rerun::clap::RerunArgs,
}

fn main() -> anyhow::Result<()> {
    re_log::setup_native_logging();
    let args = Args::parse();
    args.rerun
        .clone()
        .run("mesh_over_custom_time", true, move |rec| {
            run(&rec).unwrap();
        })
}

fn run(rec: &RecordingStream) -> anyhow::Result<()> {
    rec.log_timeless("spheres", &rerun::ViewCoordinates::RIGHT_HAND_Y_UP)?;

    rec.log_timeless(
        "LOD_10",
        &rerun::Transform3D::from_translation((-100.0, 0.0, 0.0)),
    )?;
    rec.log_timeless(
        "LOD_1000",
        &rerun::Transform3D::from_translation((100.0, 0.0, 0.0)),
    )?;

    // 600 vertices per frame
    for i in 0..10000 {
        rec.set_time_sequence("frame", i);

        let radius = i as f32 / 10000.0 * 50.0 + 0.1;

        let (vertices, normals) = generate_sphere_mesh_vertices(radius, 10);
        rec.log(
            "LOD_10",
            &Mesh3D::new(vertices).with_vertex_normals(normals),
        )?;

        // 60k vertices per frame
        if i % 10 == 0 {
            let (vertices, normals) = generate_sphere_mesh_vertices(radius, 100);
            rec.log(
                "LOD_100",
                &Mesh3D::new(vertices).with_vertex_normals(normals),
            )?;
        }

        // 6M vertices per frame
        if i % 100 == 0 {
            let (vertices, normals) = generate_sphere_mesh_vertices(radius, 1000);
            rec.log(
                "LOD_1000",
                &Mesh3D::new(vertices).with_vertex_normals(normals),
            )?;
        }
    }

    Ok(())
}

fn generate_sphere_mesh_vertices(
    radius: f32,
    num_points: usize,
) -> (Vec<glam::Vec3>, Vec<glam::Vec3>) {
    let mut vertices = Vec::with_capacity(num_points * num_points * 3 * 2);
    let mut normals = Vec::with_capacity(num_points * num_points * 3 * 2);

    let phi_step = std::f32::consts::PI / (num_points as f32);
    let theta_step = 2.0 * std::f32::consts::PI / (num_points as f32);

    for i in 0..num_points {
        let phi = i as f32 * phi_step;
        let next_phi = (i + 1) as f32 * phi_step;

        for j in 0..num_points {
            let theta = j as f32 * theta_step;
            let next_theta = (j + 1) as f32 * theta_step;

            let vertex1 = calculate_vertex(radius, phi, theta);
            let vertex2 = calculate_vertex(radius, next_phi, theta);
            let vertex3 = calculate_vertex(radius, phi, next_theta);
            let vertex4 = calculate_vertex(radius, next_phi, next_theta);

            let normal1 = calculate_normal(vertex2, vertex1, vertex3);
            let normal2 = calculate_normal(vertex4, vertex2, vertex3);

            vertices.push(vertex1);
            vertices.push(vertex2);
            vertices.push(vertex3);
            normals.push(normal1);
            normals.push(normal1);
            normals.push(normal1);

            vertices.push(vertex2);
            vertices.push(vertex4);
            vertices.push(vertex3);
            normals.push(normal2);
            normals.push(normal2);
            normals.push(normal2);
        }
    }

    (vertices, normals)
}

fn calculate_vertex(radius: f32, phi: f32, theta: f32) -> glam::Vec3 {
    let x = radius * phi.sin() * theta.cos();
    let y = radius * phi.sin() * theta.sin();
    let z = radius * phi.cos();

    glam::Vec3::new(x, y, z)
}

fn calculate_normal(v1: glam::Vec3, v2: glam::Vec3, v3: glam::Vec3) -> glam::Vec3 {
    let edge1 = v2 - v1;
    let edge2 = v3 - v1;

    edge1.cross(edge2).normalize()
}
