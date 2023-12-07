use axum::http;
use tower::ServiceBuilder;


fn testtt() {
    use tower_http::compression::{
        Compression,
        predicate::{Predicate, NotForContentType, DefaultPredicate},
    };
    use tower::util::service_fn;
    
    // Placeholder service_fn
    let service = service_fn(|_: ()| async {
        Ok::<_, std::io::Error>(http::Response::new(()))
    });
    
    // build our custom compression predicate
    // its recommended to still include `DefaultPredicate` as part of
    // custom predicates
    let predicate = DefaultPredicate::new()
        // don't compress responses who's `content-type` starts with `application/json`
        .and(NotForContentType::new("application/json"));
    
    let service = Compression::new(service).compress_when(predicate)
    let service = ServiceBuilder::new().layer(service);
}