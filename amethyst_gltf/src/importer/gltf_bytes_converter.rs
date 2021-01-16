use gltf::Document;
use amethyst_assets::error::Error;

pub fn convert_bytes(bytes: &Vec<u8>) -> Result<(Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>), Error> {
    log::debug!("Starting Gltf import");
    if bytes.starts_with(b"glTF") {
        log::debug!("Importing as a .glb");
        convert_glb(&bytes)
    } else {
        log::debug!("Importing as a standard .gltf");
        convert_standard(&bytes)
    }
}

fn convert_standard(
    bytes: &Vec<u8>,
) -> Result<(Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>), Error> {
    let result  = gltf::import_slice(&bytes.as_slice());
    log::debug!("Standard import slice has been done with result {:?}", result);
    if let Err(err) = result {
        log::error!("GLTF Standard import error: {:?}", err);
        return Err(Error::Boxed(Box::new(err)));
    }
    Ok(result.unwrap())
}

fn convert_glb(
    data: &Vec<u8>,
) -> Result<(Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>), Error> {
    let result  = gltf::import_slice(&data);
    if let Err(err) = result {
        log::error!("GLTF Import error: {:?}", err);
        return Err(Error::Boxed(Box::new(err)));
    }
    Ok(result.unwrap())
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::GltfSceneOptions;
    use std::fs::File;
    use std::io::Read;
    use atelier_assets::importer::BoxedImporter;
    use type_uuid::TypeUuid;


    #[test]
    fn should_import_glb_gltf() {
        let mut f = File::open("test/suzanne.glb").expect("suzanne.glb not found");
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer).expect("read_to_end did not work");
        let gltf_import = convert_glb(&buffer);
        match gltf_import {
            Ok(gltf) => {
                let doc = gltf.0;
                assert_eq!(2, doc.images().len());
                assert_eq!(1, doc.materials().len());
                assert_eq!(1, doc.meshes().len());
                assert_eq!(1, doc.nodes().len());
                assert_eq!(1, doc.scenes().len());
                assert_eq!(2, doc.textures().len());
            },
            Err(e) => {
                panic!("Error during gltf import {:?}", e)
            }
        }
    }
}