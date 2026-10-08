use bollard::{Docker, query_parameters::ListImagesOptionsBuilder};

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_podman_defaults().unwrap();
    let version = docker.version().await.unwrap();
    println!("Version: {:#?}", version);
    let options = ListImagesOptionsBuilder::default().all(true).build();
    let images = &docker.list_images(Some(options)).await.unwrap();

    for image in images {
        println!("-> {:?}", image);
    }
}
