// this is full of sooo many hacks and engine-specific limitations...
// generating type-safe bindings is quite hard =(

use std::{
    collections::{BTreeMap, HashMap},
    num::NonZeroU32,
    path::Path,
};

use heck::ToPascalCase;
use indexmap::IndexMap;
use itertools::{EitherOrBoth, Itertools};
use naga::{
    Handle, ShaderStage, Type, UniqueArena,
    valid::{Capabilities, ValidationFlags},
};
use quote::{TokenStreamExt, quote};
use shin_render_shader_types::{
    uniforms::{
        ClearUniformParams, FillUniformParams, FontBorderUniformParams, FontUniformParams,
        LayerUniformParams, MovieUniformParams, SpriteUniformParams, UniformType,
        WiperDefaultUniformParams, WiperMaskUniformParams,
        metadata::{ArraySchema, PrimitiveType, StructSchema, TypeSchema},
    },
    vertices::{
        BlendVertex, LayerVertex, MaskVertex, MovieVertex, PosColTexVertex, PosColVertex,
        PosVertex, TextVertex, VertexType, WindowVertex,
    },
};

fn gen_primitive(primitive_type: PrimitiveType) -> naga::TypeInner {
    match primitive_type {
        PrimitiveType::Float32 => naga::TypeInner::Scalar(naga::Scalar::F32),
        PrimitiveType::Float32x2 => naga::TypeInner::Vector {
            size: naga::VectorSize::Bi,
            scalar: naga::Scalar::F32,
        },
        PrimitiveType::Float32x3 => naga::TypeInner::Vector {
            size: naga::VectorSize::Tri,
            scalar: naga::Scalar::F32,
        },
        PrimitiveType::Float32x4 => naga::TypeInner::Vector {
            size: naga::VectorSize::Quad,
            scalar: naga::Scalar::F32,
        },
        PrimitiveType::Float32x4x4 => naga::TypeInner::Matrix {
            columns: naga::VectorSize::Quad,
            rows: naga::VectorSize::Quad,
            scalar: naga::Scalar::F32,
        },
        PrimitiveType::Uint32 => naga::TypeInner::Scalar(naga::Scalar::U32),
    }
}

fn populate_primitive_map(
    type_arena: &mut UniqueArena<Type>,
) -> HashMap<PrimitiveType, Handle<Type>> {
    let mut primitives = HashMap::new();

    for prim in enum_iterator::all::<PrimitiveType>() {
        let ty = Type {
            name: None,
            inner: gen_primitive(prim),
        };
        let handle = type_arena.insert(ty, naga::Span::UNDEFINED);

        primitives.insert(prim, handle);
    }

    primitives
}

struct KnownStructInfo {
    schema: StructSchema,
    handle: Handle<Type>,
    fully_qualified_rust_name: String,
}

struct GenCtx {
    type_arena: UniqueArena<Type>,
    layouter: naga::proc::Layouter,
    primitives: HashMap<PrimitiveType, Handle<Type>>,
    known_vertices: HashMap<String, String>,
    known_structs: HashMap<String, KnownStructInfo>,
}

impl GenCtx {
    fn new() -> Self {
        let mut type_arena = UniqueArena::new();
        let primitives = populate_primitive_map(&mut type_arena);

        Self {
            type_arena,
            layouter: naga::proc::Layouter::default(),
            primitives,
            known_vertices: HashMap::new(),
            known_structs: HashMap::new(),
        }
    }

    fn gen_array(&mut self, schema: &ArraySchema) -> Handle<Type> {
        let ty = match schema.ty {
            TypeSchema::Primitive(prim) => *self.primitives.get(prim).unwrap(),
            TypeSchema::Struct(_schema) => {
                todo!("Structs in arrays??? Who would need such complexity?????")
                // self.gen_struct(schema)
            }
            TypeSchema::Array(_array) => {
                todo!("Arrays in arrays??? Who would need such complexity?????")
            }
        };

        let ty = Type {
            name: None,
            inner: naga::TypeInner::Array {
                base: ty,
                size: naga::ArraySize::Constant(NonZeroU32::new(schema.length).unwrap()),
                stride: schema.stride,
            },
        };
        self.type_arena.insert(ty, naga::Span::UNDEFINED)
    }

    fn gen_struct(
        &mut self,
        schema: &StructSchema,
        fully_qualified_rust_name: &str,
    ) -> Handle<Type> {
        if let Some(KnownStructInfo {
            schema: existing_schema,
            handle,
            ..
        }) = self.known_structs.get(schema.name)
        {
            assert_eq!(existing_schema, schema);
            return *handle;
        };

        let members = schema
            .fields
            .iter()
            .map(|f| {
                let ty = match f.ty {
                    TypeSchema::Primitive(prim) => *self.primitives.get(prim).unwrap(),
                    TypeSchema::Struct(_schema) => {
                        todo!("Structs in structs??? Who would need such complexity?????")
                        // self.gen_struct(schema)
                    }
                    TypeSchema::Array(array) => self.gen_array(array),
                };

                naga::StructMember {
                    name: Some(f.name.to_string()),
                    ty,
                    binding: None,
                    offset: f.offset,
                }
            })
            .collect();

        let ty = Type {
            name: Some(schema.name.to_string()),
            inner: naga::TypeInner::Struct {
                members,
                span: schema.size,
            },
        };
        let handle = self.type_arena.insert(ty, naga::Span::UNDEFINED);

        self.known_structs
            .insert(schema.name.to_string(), KnownStructInfo {
                schema: *schema,
                handle,
                fully_qualified_rust_name: fully_qualified_rust_name.to_string(),
            });

        handle
    }

    fn gen_vertex_impl(
        &mut self,
        descritor: &wgpu::VertexBufferLayout,
        names: &[&str],
    ) -> naga::TypeInner {
        let mut members = Vec::with_capacity(descritor.attributes.len());

        let gctx = naga::proc::GlobalCtx {
            types: &self.type_arena,
            constants: &Default::default(),
            overrides: &Default::default(),
            global_expressions: &Default::default(),
        };

        self.layouter.update(gctx).unwrap();

        let mut offset = 0;
        for (attribute, name) in std::iter::zip(descritor.attributes, names) {
            let ty = match attribute.format {
                wgpu::VertexFormat::Unorm8x4 => PrimitiveType::Float32x4,
                wgpu::VertexFormat::Float32 => PrimitiveType::Float32,
                wgpu::VertexFormat::Float32x2 => PrimitiveType::Float32x2,
                wgpu::VertexFormat::Float32x3 => PrimitiveType::Float32x3,
                wgpu::VertexFormat::Float32x4 => PrimitiveType::Float32x4,

                _ => todo!(
                    "unsupported vertex format {:?} for attribute {}",
                    attribute.format,
                    name
                ),
            };
            let ty = *self.primitives.get(&ty).unwrap();
            let member_layout = self.layouter[ty];

            offset = member_layout.alignment.round_up(offset);

            members.push(naga::StructMember {
                name: Some(name.to_string()),
                ty,
                binding: Some(naga::Binding::Location {
                    location: attribute.shader_location,
                    second_blend_source: false,
                    interpolation: None,
                    sampling: None,
                }),
                // NOTE: this offset is not the same as the offset specified in the VertexBufferLayout
                // this is because the layout of vertex stream and vertex the wgsl struct, while similar, are not the same
                // this is due to lack of paddings in the vertex stream, as well as the conversion of UNORM attributes to float attributes
                offset,
            });

            offset += member_layout.size;
        }

        naga::TypeInner::Struct {
            members,
            span: offset,
        }
    }

    fn gen_vertex<T: VertexType>(&mut self) {
        // here be dragons: here we rely on the fact that the name returned by the type_name will be usable in rust to name the same type
        // the docs make it abundantly clear that this is not guaranteed, but it works for now
        let fully_qualified_rust_name = std::any::type_name::<T>();

        let ty = self.gen_vertex_impl(&T::DESCRIPTOR, T::ATTRIBUTE_NAMES);
        let ty = Type {
            name: Some(T::NAME.to_string()),
            inner: ty,
        };
        self.type_arena.insert(ty, naga::Span::UNDEFINED);

        self.known_vertices
            .insert(T::NAME.to_string(), fully_qualified_rust_name.to_string());
    }
    fn gen_uniform<T: UniformType>(&mut self) -> Handle<Type> {
        // here be dragons: here we rely on the fact that the name returned by the type_name will be usable in rust to name the same type
        // the docs make it abundantly clear that this is not guaranteed, but it works for now
        let fully_qualified_rust_name = std::any::type_name::<T>();

        match T::SCHEMA {
            TypeSchema::Primitive(prim) => *self.primitives.get(&prim).unwrap(),
            TypeSchema::Struct(schema) => self.gen_struct(&schema, fully_qualified_rust_name),
            TypeSchema::Array(array) => self.gen_array(&array),
        }
    }
}

struct WgslSchema {
    module_source: String,
    vertex_rust_names: HashMap<String, String>,
    struct_rust_names: HashMap<String, String>,
}

fn generate_wgsl_types() -> WgslSchema {
    let mut ctx = GenCtx::new();

    ctx.gen_vertex::<PosVertex>();
    ctx.gen_vertex::<PosColVertex>();
    ctx.gen_vertex::<PosColTexVertex>();
    ctx.gen_vertex::<TextVertex>();
    ctx.gen_vertex::<BlendVertex>();
    ctx.gen_vertex::<WindowVertex>();
    ctx.gen_vertex::<LayerVertex>();
    ctx.gen_vertex::<MaskVertex>();
    ctx.gen_vertex::<MovieVertex>();

    ctx.gen_uniform::<ClearUniformParams>();
    ctx.gen_uniform::<FillUniformParams>();
    ctx.gen_uniform::<SpriteUniformParams>();
    ctx.gen_uniform::<FontUniformParams>();
    ctx.gen_uniform::<FontBorderUniformParams>();
    ctx.gen_uniform::<LayerUniformParams>();
    ctx.gen_uniform::<MovieUniformParams>();
    ctx.gen_uniform::<WiperDefaultUniformParams>();
    ctx.gen_uniform::<WiperMaskUniformParams>();

    let vertex_rust_names = ctx.known_vertices;
    let struct_rust_names = ctx
        .known_structs
        .into_iter()
        .map(|(k, v)| (k, v.fully_qualified_rust_name))
        .collect::<HashMap<_, _>>();

    let module = naga::Module {
        types: ctx.type_arena,
        ..naga::Module::default()
    };

    let info = naga::valid::Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap();
    let module_source = naga::back::wgsl::write_string(
        &module,
        &info,
        naga::back::wgsl::WriterFlags::EXPLICIT_TYPES,
    )
    .unwrap();

    WgslSchema {
        module_source,
        vertex_rust_names,
        struct_rust_names,
    }
}

struct ModuleInfo {
    source: String,
    is_entrypoint: bool,
    dependencies: Vec<String>,
}

fn add_module_to_composer(
    directory: &str,
    composer: &mut naga_oil::compose::Composer,
    module_infos: &BTreeMap<String, ModuleInfo>,
    name: &str,
) {
    let ModuleInfo {
        source,
        is_entrypoint: _,
        dependencies,
    } = module_infos
        .get(name)
        .unwrap_or_else(|| panic!("missing module {}", name));

    for dep in dependencies {
        if !composer.module_sets.contains_key(dep) {
            add_module_to_composer(directory, composer, module_infos, dep);
        }
    }

    composer
        .add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
            source,
            file_path: &format!("{}/{}.wgsl", directory, name),
            language: naga_oil::compose::ShaderLanguage::Wgsl,
            as_name: Some(name.to_string()),
            additional_imports: &[],
            shader_defs: Default::default(),
        })
        .unwrap();
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ShaderBindingGroupDescriptor {
    Texture {
        texture_binding: u32,
        sampler_binding: u32,
    },
    Uniform {
        binding: u32,
        ty: String,
        size: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShaderDescriptor {
    vertex_type: String,
    bind_groups: IndexMap<String, ShaderBindingGroupDescriptor>,
    vertex_entry_name: String,
    fragment_entry_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShaderWithDescriptor {
    snake_name: String,
    pascal_name: String,
    descriptor: ShaderDescriptor,
    wgsl: String,
    spirv: Vec<u32>,
}

fn find_entrypoints(wgsl_dir: &Path, wgsl_schema: &WgslSchema) -> Vec<ShaderWithDescriptor> {
    let mut sources = HashMap::new();

    for entry in std::fs::read_dir(wgsl_dir).unwrap() {
        let entry = entry.unwrap();

        if entry.path().extension().and_then(|v| v.to_str()) == Some("wgsl")
            && entry.file_type().unwrap().is_file()
        {
            let path = entry.path();

            if path.file_stem().and_then(|v| v.to_str()) != Some("types") {
                println!("cargo:rerun-if-changed={}", path.display());
            }

            let source = std::fs::read_to_string(&path).unwrap();

            let name = path.file_stem().unwrap().to_str().unwrap();

            sources.insert(name.to_string(), source);
        }
    }

    let mut module_infos = BTreeMap::new();

    for (name, source) in sources {
        let (self_proclaimed_name, imports, defines) =
            naga_oil::compose::get_preprocessor_data(&source);

        if self_proclaimed_name.is_some() {
            panic!(
                "shaders should not define their names, but found one in {}",
                name
            );
        }
        if !defines.is_empty() {
            panic!(
                "shaders should not define any defines, but found some in {}",
                name
            );
        }

        // this sucks... we need a real parser supporting extensions, but naga_oil is also written using regexes...
        let is_entrypoint = source.contains("@vertex") || source.contains("@fragment");

        let module_info = ModuleInfo {
            is_entrypoint,
            dependencies: imports.iter().map(|v| v.import.clone()).collect(),
            source,
        };

        module_infos.insert(name, module_info);
    }

    let types_suffix = naga_oil::compose::Composer::decorated_name(Some("types"), "");

    let mut result = Vec::new();

    let mut composer = naga_oil::compose::Composer::default();
    for (name, module_info) in &module_infos {
        if module_info.is_entrypoint {
            for dep in &module_info.dependencies {
                // it's a DAG!
                // find the right topological order to insert the shaders into the composer
                add_module_to_composer(
                    wgsl_dir.to_str().unwrap(),
                    &mut composer,
                    &module_infos,
                    dep,
                );
            }

            let module = composer
                .make_naga_module(naga_oil::compose::NagaModuleDescriptor {
                    source: &module_info.source,
                    file_path: &format!("{}/{}.wgsl", wgsl_dir.to_str().unwrap(), name),
                    shader_type: naga_oil::compose::ShaderType::Wgsl,
                    ..Default::default()
                })
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to make naga module for {}: {:#?}",
                        name,
                        // anyhow::Error::new(error),
                        error,
                    )
                });

            let module_info =
                naga::valid::Validator::new(ValidationFlags::all(), Capabilities::empty())
                    .validate(&module)
                    .unwrap();

            let module_source = naga::back::wgsl::write_string(
                &module,
                &module_info,
                naga::back::wgsl::WriterFlags::EXPLICIT_TYPES,
            )
            .unwrap();

            let bounds_check_policy = naga::proc::BoundsCheckPolicy::Restrict;

            let module_spirv = naga::back::spv::write_vec(
                &module,
                &module_info,
                &naga::back::spv::Options {
                    lang_version: (1, 0),
                    flags: naga::back::spv::WriterFlags::DEBUG
                        | naga::back::spv::WriterFlags::ADJUST_COORDINATE_SPACE
                        | naga::back::spv::WriterFlags::LABEL_VARYINGS
                        | naga::back::spv::WriterFlags::CLAMP_FRAG_DEPTH,
                    binding_map: naga::back::spv::BindingMap::default(),
                    capabilities: Some(naga::FastHashSet::from_iter([
                        spirv::Capability::Matrix,
                        spirv::Capability::Shader,
                    ])),
                    bounds_check_policies: naga::proc::BoundsCheckPolicies {
                        index: bounds_check_policy,
                        buffer: bounds_check_policy,
                        image_load: bounds_check_policy,
                        binding_array: bounds_check_policy,
                    },
                    zero_initialize_workgroup_memory:
                        naga::back::spv::ZeroInitializeWorkgroupMemoryMode::Polyfill,
                    debug_info: None,
                },
                None,
            )
            .unwrap();

            // eprintln!("module:\n{}", module_source);

            let mut fragment_entry_name = None;
            let mut vertex_entry_name = None;
            let mut vertex_type_name = None;
            for entry_point in &module.entry_points {
                match entry_point.stage {
                    ShaderStage::Fragment => {
                        assert_eq!(fragment_entry_name, None);
                        fragment_entry_name = Some(entry_point.name.clone());
                    }
                    ShaderStage::Vertex => {
                        assert_eq!(vertex_entry_name, None);
                        vertex_entry_name = Some(entry_point.name.clone());

                        // TODO: support built-in input values if/when we need them
                        assert_eq!(entry_point.function.arguments.len(), 1);
                        let argument = &entry_point.function.arguments[0];
                        assert_eq!(argument.binding, None);
                        let type_name = module.types[argument.ty].name.clone().unwrap();
                        let type_name = type_name.strip_suffix(&types_suffix).unwrap();

                        vertex_type_name =
                            Some(wgsl_schema.vertex_rust_names.get(dbg!(type_name)).unwrap());
                    }
                    ShaderStage::Compute => {
                        panic!("compute shaders are not supported")
                    }
                }
            }

            let fragment_entry_name = fragment_entry_name.unwrap();
            let vertex_entry_name = vertex_entry_name.unwrap();
            let vertex_type_name = vertex_type_name.unwrap();

            struct StructBindingInfo {
                binding: u32,
                fully_qualified_rust_ty: String,
                size: u32,
            }

            let mut texture_bindings = IndexMap::<String, u32>::new();
            let mut sampler_bindings = IndexMap::<String, u32>::new();
            let mut struct_bindings = IndexMap::<String, StructBindingInfo>::new();

            for (_, var) in module.global_variables.iter() {
                let Some(binding) = &var.binding else {
                    continue;
                };

                assert_eq!(binding.group, 0);

                let ty = &module.types[var.ty];
                #[expect(clippy::bool_assert_comparison)]
                match &ty.inner {
                    &naga::TypeInner::Image {
                        dim,
                        arrayed,
                        class,
                    } => {
                        let name = var
                            .name
                            .as_deref()
                            .unwrap()
                            .strip_suffix("_texture")
                            .unwrap();
                        // assert_eq!(binding.binding, 0);
                        assert_eq!(arrayed, false);
                        assert_eq!(dim, naga::ImageDimension::D2);
                        assert_eq!(class, naga::ImageClass::Sampled {
                            kind: naga::ScalarKind::Float,
                            multi: false
                        });

                        texture_bindings.insert(name.to_string(), binding.binding);
                    }
                    &naga::TypeInner::Sampler { comparison } => {
                        let name = var
                            .name
                            .as_deref()
                            .unwrap()
                            .strip_suffix("_sampler")
                            .unwrap();
                        // assert_eq!(binding.binding, 1);
                        assert_eq!(comparison, false);

                        sampler_bindings.insert(name.to_string(), binding.binding);
                    }
                    naga::TypeInner::Struct { .. } => {
                        // assert_eq!(binding.binding, 0);

                        let type_name = ty
                            .name
                            .as_ref()
                            .unwrap()
                            .strip_suffix(&types_suffix)
                            .unwrap();

                        let fully_qualified_rust_name =
                            wgsl_schema.struct_rust_names.get(type_name).unwrap();

                        struct_bindings.insert(var.name.clone().unwrap(), StructBindingInfo {
                            binding: binding.binding,
                            fully_qualified_rust_ty: fully_qualified_rust_name.clone(),
                            size: ty.inner.size(module.to_ctx()),
                        });
                    }
                    e => panic!("unsupported global variable type {:?}", e),
                }
            }

            // eprintln!("texture_bindings: {:?}", texture_bindings);
            // eprintln!("sampler_bindings: {:?}", sampler_bindings);
            // eprintln!("struct_bindings: {:?}", struct_bindings);

            let mut bindings_unified = IndexMap::new();

            for (name, info) in struct_bindings.into_iter() {
                assert_eq!(
                    bindings_unified.insert(name, ShaderBindingGroupDescriptor::Uniform {
                        binding: info.binding,
                        ty: info.fully_qualified_rust_ty,
                        size: info.size,
                    },),
                    None
                );
            }

            for v in texture_bindings
                .into_iter()
                .merge_join_by(sampler_bindings, |(texture_name, _), (sampler_name, _)| {
                    texture_name.cmp(sampler_name)
                })
            {
                match v {
                    EitherOrBoth::Both((name, texture_binding), (_, sampler_binding)) => {
                        let existing =
                            bindings_unified.insert(name, ShaderBindingGroupDescriptor::Texture {
                                texture_binding,
                                sampler_binding,
                            });
                        assert_eq!(existing, None);
                    }
                    EitherOrBoth::Left((texture_name, _)) => {
                        panic!("missing sampler for texture {texture_name}")
                    }
                    EitherOrBoth::Right((sampler_name, _)) => {
                        panic!("missing texture for sampler {sampler_name}")
                    }
                }
            }

            // if let Some(max_binding) = bindings_unified
            //     .iter()
            //     .map(|(_, d)| match d {
            //         &ShaderBindingGroupDescriptor::Texture {
            //             texture_binding,
            //             sampler_binding,
            //         } => std::cmp::max(texture_binding, sampler_binding),
            //         &ShaderBindingGroupDescriptor::Uniform { binding, .. } => binding,
            //     })
            //     .max()
            // {
            //     // TODO: this check is invalid
            //     // assert_eq!(
            //     //     max_binding as usize,
            //     //     bindings_unified.len() - 1,
            //     //     "binding numbers must not have gaps"
            //     // );
            // }

            // eprintln!("bindings_unified: {:?}", bindings_unified);

            // sort by binding number, so just for nicer output
            bindings_unified.sort_by_cached_key(|_, v| match *v {
                ShaderBindingGroupDescriptor::Texture {
                    texture_binding,
                    sampler_binding,
                } => std::cmp::min(texture_binding, sampler_binding),
                ShaderBindingGroupDescriptor::Uniform { binding, .. } => binding,
            });

            let descriptor = ShaderDescriptor {
                vertex_type: vertex_type_name.clone(),
                bind_groups: bindings_unified,
                vertex_entry_name,
                fragment_entry_name,
            };

            // eprintln!("descriptor: {:?}", descriptor);

            result.push(ShaderWithDescriptor {
                snake_name: name.clone(),
                pascal_name: name.to_pascal_case(),
                descriptor,
                wgsl: module_source,
                spirv: module_spirv,
            });
            // eprintln!("module: {:?}", module);
        }
    }

    result
}

fn codegen_shader_descriptor(shader: &ShaderWithDescriptor) -> proc_macro2::TokenStream {
    let snake_name = &shader.snake_name;

    let spirv = {
        let mut res = proc_macro2::TokenStream::new();
        res.append_separated(
            &shader.spirv,
            proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone),
        );
        res
    };
    let wgsl = &shader.wgsl;

    let ShaderDescriptor {
        vertex_type: _,
        bind_groups,
        vertex_entry_name,
        fragment_entry_name,
    } = &shader.descriptor;

    let bind_groups = bind_groups
        .iter()
        .map(|(_name, bind_group)| match bind_group {
            ShaderBindingGroupDescriptor::Texture { texture_binding, sampler_binding } => {
                quote!(crate::ShaderBindingGroupDescriptor::Texture { texture_binding: #texture_binding, sampler_binding: #sampler_binding },)
            }
            ShaderBindingGroupDescriptor::Uniform { binding, size, .. } => {
                quote!(crate::ShaderBindingGroupDescriptor::Uniform { binding: #binding, size: #size },)
            }
        })
        .collect::<proc_macro2::TokenStream>();

    quote! {
        crate::ShaderDescriptor {
            name: #snake_name,
            // wasm32 never supports spir-v, so don't include it
            #[cfg(not(target_arch = "wasm32"))]
            spirv: &[#spirv],
            // always include wgsl as a fallback for devices that don't support spir-v (don't have vulkan support)
            wgsl: #wgsl,
            vertex_entry: #vertex_entry_name,
            fragment_entry: #fragment_entry_name,
            bind_groups: &[
                #bind_groups
            ],
            vertex_buffer_layout: <Self::Vertex as shin_render_shader_types::vertices::VertexType>::DESCRIPTOR,
        }
    }
}

fn codegen_bindings(
    bindings_name: &proc_macro2::Ident,
    bindings: &IndexMap<String, ShaderBindingGroupDescriptor>,
) -> proc_macro2::TokenStream {
    let body = bindings
        .iter()
        .map(|(name, binding)| {
            let name = quote::format_ident!("{}", name);

            let ty = match binding {
                ShaderBindingGroupDescriptor::Texture { .. } => {
                    quote!(shin_render_shader_types::texture::TextureSource<'a>)
                }
                ShaderBindingGroupDescriptor::Uniform { ty, .. } => {
                    let ty: proc_macro2::TokenStream = ty.parse().unwrap();
                    quote!(&'a #ty)
                }
            };

            quote! {
                pub #name: #ty,
            }
        })
        .collect::<proc_macro2::TokenStream>();

    quote! {
        pub struct #bindings_name<'a> {
            #body
        }
    }
}

fn codegen_set_bindings(
    bindings: &IndexMap<String, ShaderBindingGroupDescriptor>,
) -> proc_macro2::TokenStream {
    let mut prelude = proc_macro2::TokenStream::new();

    let bind_group_entries = bindings
        .iter()
        .map(|(name, binding)| {
            let name = quote::format_ident!("{}", name);

            match binding {
                ShaderBindingGroupDescriptor::Texture {
                    texture_binding,
                    sampler_binding,
                } => {
                    quote!(
                        wgpu::BindGroupEntry {
                            binding: #texture_binding,
                            resource: wgpu::BindingResource::TextureView(bindings.#name.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: #sampler_binding,
                            resource: wgpu::BindingResource::Sampler(
                                sampler_store.get(bindings.#name.sampler)
                            ),
                        },
                    )
                }
                ShaderBindingGroupDescriptor::Uniform { binding, .. } => {
                    prelude.extend(quote! {
                        let #name = dynamic_buffer.get_uniform_with_data(bindings.#name);
                    });

                    quote! {
                        wgpu::BindGroupEntry {
                            binding: #binding,
                            resource: wgpu::BindingResource::Buffer(#name.as_buffer_binding()),
                        },
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>();

    quote! {
        fn set_bindings(
            device: &wgpu::Device,
            dynamic_buffer: &mut impl crate::DynamicBufferBackend,
            #[allow(unused)] // sampler_store may end up being unused for shaders without texture bindings
            sampler_store: &crate::TextureSamplerStore,
            bind_group_layout: &wgpu::BindGroupLayout,
            render_pass: &mut wgpu::RenderPass,
            bindings: Self::Bindings<'_>,
        ) {
            #prelude

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[#bind_group_entries],
            });
            render_pass.set_bind_group(0, &bind_group, &[]);
        }
    }
}

fn codegen_shader(shader: &ShaderWithDescriptor) -> proc_macro2::TokenStream {
    let pascal_case_name = &shader.pascal_name;

    let shader_ty_name = quote::format_ident!("{}", pascal_case_name);
    let bindings_ty_name = quote::format_ident!("{}Bindings", pascal_case_name);

    let shader_descriptor = codegen_shader_descriptor(shader);

    let bindings = codegen_bindings(&bindings_ty_name, &shader.descriptor.bind_groups);
    let vertex = shader
        .descriptor
        .vertex_type
        .parse::<proc_macro2::TokenStream>()
        .unwrap();

    let set_bindings = codegen_set_bindings(&shader.descriptor.bind_groups);

    quote! {
        #bindings

        pub struct #shader_ty_name;

        impl crate::Shader for #shader_ty_name {
            const NAME: ShaderName = ShaderName::#shader_ty_name;
            const DESCRIPTOR: crate::ShaderDescriptor = #shader_descriptor;

            type Bindings<'a> = #bindings_ty_name<'a>;
            type Vertex = #vertex;

            #set_bindings
        }
    }
}

fn codegen_shaders(shaders: &[ShaderWithDescriptor]) -> proc_macro2::TokenStream {
    let shader_names = shaders
        .iter()
        .map(|shader| {
            let name = quote::format_ident!("{}", shader.pascal_name);
            quote! {
                #name,
            }
        })
        .collect::<proc_macro2::TokenStream>();
    let shader_name_descriptors = shaders
        .iter()
        .map(|shader| {
            let name = quote::format_ident!("{}", shader.pascal_name);
            quote! {
                ShaderName::#name => <#name as crate::Shader>::DESCRIPTOR,
            }
        })
        .collect::<proc_macro2::TokenStream>();
    let shader_names = quote! {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, enum_iterator::Sequence)]
        pub enum ShaderName {
            #shader_names
        }

        impl ShaderName {
            pub fn descriptor(&self) -> crate::ShaderDescriptor {
                match self {
                    #shader_name_descriptors
                }
            }
        }
    };

    let shaders = shaders
        .iter()
        .map(codegen_shader)
        .collect::<proc_macro2::TokenStream>();

    quote! {
        #shader_names

        #shaders
    }
}

fn codegen_shaders_file(shaders: &[ShaderWithDescriptor]) -> String {
    let shaders_tokens = codegen_shaders(shaders);

    eprintln!("{}", shaders_tokens);

    let shaders_file = syn::parse2(shaders_tokens).unwrap();

    prettyplease::unparse(&shaders_file)
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(manifest_dir.as_str());
    let wgsl_dir = manifest_dir.join("wgsl");

    // step 1.
    // traverse the types and check that the on-disk types.wgsl is up-to-date

    let schema = generate_wgsl_types();
    std::fs::write(wgsl_dir.join("types.wgsl"), &schema.module_source)
        .expect("Failed to write types.wgsl");

    // step 2.
    // traverse the shaders and generate the bindings
    let shaders = find_entrypoints(&wgsl_dir, &schema);

    eprintln!("shaders: {:#?}", shaders);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out_dir.as_str());
    for shader in &shaders {
        // let wgsl_out_file = out_dir.join(&shader.name);
        let spirv_out_file = out_dir.join(format!("{}.spv", shader.snake_name));
        std::fs::write(spirv_out_file, bytemuck::cast_slice::<_, u8>(&shader.spirv)).unwrap()
    }

    std::fs::write(out_dir.join("shaders.rs"), codegen_shaders_file(&shaders)).unwrap();

    // todo!()
}
