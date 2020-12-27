use core::any::Any;
use std::collections::HashMap;
use core::any::TypeId;
/**
* to compile, rustc --edition=2018 asset_loader.rs && ./asset_loader
*
* See fn main() for usage
* Centralized resource storage & loading
* Take an existing mesh/shader/resources and transform & store them inside a centralized map
*
* I have no idea whether this is considered idiomatic or not,
* People have recommended to use enum instead of Any
*
* Though, the advantage of this method is you can easily extend with new types 
* as long as you implement LoaderRouter with route_loader!() and Loader<T, R> manually
**/

macro_rules! route_loader {
  ($loader_ident:ident, $struct_ident:ident) => {
    impl LoaderRouter for $loader_ident {
        fn enroute_create(&self, any : &dyn Any) -> Box<dyn Any> {
            Box::new(self.create(any.downcast_ref::<$struct_ident>().unwrap()))
        }
    }
  };
}


trait LoaderRouter {
    fn enroute_create(&self, any :  &dyn Any) -> Box<dyn Any>;
}

trait Loader<T, R> {
    fn create(&self, t : &T) -> R;

    // can add other functions such as update or delete
}

struct Mesh(String);
struct Shader(String);

struct OpenGLMesh(String);
struct OpenGLShader(String);

struct MeshLoader;
impl Loader<Mesh, OpenGLMesh> for MeshLoader {
    fn create(&self, t : &Mesh) -> OpenGLMesh{
        OpenGLMesh(format!("{} OpenGL", t.0))
    }
}
route_loader!(MeshLoader, Mesh);

struct ShaderLoader;
impl Loader<Shader, OpenGLShader> for ShaderLoader {
    fn create(&self, t : &Shader) -> OpenGLShader {
        OpenGLShader(format!("{} OpenGL", t.0))
    }
}
route_loader!(ShaderLoader, Shader);

// You can add another type to be stored on resource pile 
// as long as you implement Loader and route_loader!() like above.
/**
struct MyResourceLoader;
impl Loader<MyResource, MyTransformedResource> for MyResourceLoader {
    fn create(&self, t : &MyResource) -> MyTransformedResource {
        MyTransformedResource
    }
}
route_loader!(MyResourceLoader, MyResource);
**/


struct ResourcePile {
    resources: HashMap<TypeId, HashMap<i32, Box<dyn Any>>>,

    loaders : HashMap<TypeId, Box<dyn LoaderRouter>>,

    counter: i32,
}

impl ResourcePile {
    pub fn new() -> ResourcePile {
        ResourcePile {
            resources: HashMap::new(),
            loaders: HashMap::new(),
            counter: 0,
        }
    }
    
    pub fn add_resource<T : 'static>(&mut self, t: T){
        let result = self.loaders.get(&TypeId::of::<T>()).unwrap().enroute_create(&t);

        self.resources.entry(TypeId::of::<T>())
            .or_insert(HashMap::new())
            .insert(self.counter, result);

        // Use trait Identifiable or UUID on T instead of counter like this 
        self.counter += 1;
    }
    
    pub fn add_loader<R: 'static, T : 'static + LoaderRouter> (&mut self, k : T){
        self.loaders.insert(TypeId::of::<R>(), Box::new(k));
    }

    pub fn get_transformed_resource<T : 'static, R : 'static>(&self, id: i32) -> &R{
        let t = self.resources.get(&TypeId::of::<T>()).unwrap()
            .get(&id).unwrap();

        t.downcast_ref::<R>().unwrap()
    }
}


fn main() {
    let cube_mesh = Mesh("CubeMesh".to_owned());
    let sphere_mesh = Mesh("SphereMesh".to_owned());
    let pbr_shader = Shader("PbrShader".to_owned());

    let mut pile = ResourcePile::new();
    pile.add_loader::<Mesh, MeshLoader>(MeshLoader);
    pile.add_loader::<Shader, ShaderLoader>(ShaderLoader);
    
    pile.add_resource(cube_mesh);
    pile.add_resource(sphere_mesh);
    pile.add_resource(pbr_shader);

    // In real case, pass in the original resource with UUID or Identifiable trait 
    let ogl_cube = pile.get_transformed_resource::<Mesh, OpenGLMesh>(0);
    let ogl_sphere = pile.get_transformed_resource::<Mesh, OpenGLMesh>(1);
    let ogl_pbr = pile.get_transformed_resource::<Shader, OpenGLShader>(2);

    println!("{}\n{}\n{}\n", ogl_cube.0, ogl_sphere.0, ogl_pbr.0);
}
