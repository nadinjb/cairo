pub use tonic;

tonic::include_proto!("proc_macro_server.api");

#[cfg(feature = "build-server")]
pub fn reflection_service() -> tonic_reflection::server::v1::ServerReflectionServer<
    impl tonic_reflection::server::v1::ServerReflection,
> {
    tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!("descriptor"))
        .build_v1()
        .unwrap()
}
