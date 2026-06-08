# Cocos4-Rust

Rust implementation of Cocos4 game engine.

## Progress

Fully completed conversion of Cocos4 engine from C++ to Rust.

### Completed Modules (48/48 - 100%)

#### Math module
- Vec2, Vec3, Vec4, Mat3, Mat4, Quaternion, Color, Geometry

#### Base module
- Types, RefCounted, Log, Scheduler, Timer, ObjectPool, Data, Value, Util
- threading: ReadWriteLock, MessageQueue, ThreadPool, ThreadSafeCounter, AutoReleasePool

#### Core module
- SceneGraph (Node/Scene/Transform)
- Geometry: AABB, Sphere, OBB, Ray, Line, Plane, Frustum, Capsule, Triangle, Spline, AnimationCurve
- Animation, EventSystem, EventBus, EventTarget, StateMachine, SpatialGrid
- Assets: Material, Mesh, Texture, Font, Image, Effect
- memop: Pool, RecyclePool, CachedArray

#### Renderer/gfx-base
- Format (130+ variants), BufferUsage, MemoryUsage, TextureType/Usage/Flags
- Feature, ObjectType, Status, FormatFeature, ColorMask, CullMode, PolygonMode
- DynamicStateFlags, StencilFace, AccessFlags, ClearFlags, BarrierType
- DrawInfo, DispatchInfo, BufferTextureCopy, TextureBlit, TextureCopy
- DeviceCaps, Offset, Extent, TextureSubresLayers, MarkerInfo
- Buffer, Texture, Shader, Sampler, RenderPass, Framebuffer, DescriptorSet
- CommandBuffer (full API), Device (full API: create/acquire/present/flush)

#### Renderer/gfx-empty
- Null/no-op backend for testing and CI

#### Renderer/gfx-validator
- Validation/debug layer with ResourceTracker and ValidationLog

#### Renderer/gfx-agent
- Multi-threaded command recording/submission proxy layer

#### Renderer/gfx-wgpu
- WebGPU backend with native GPU resource management

#### Renderer/frame-graph
- FrameGraph: add_pass, create_texture, import_external_texture, compile, execute, reset
- ResourceNode, VirtualResource, Blackboard, PassNodeBuilder

#### Renderer/pipeline
- Forward pipeline (ForwardPipeline, ForwardFlow, ForwardStage)
- Defines, RenderFlow, RenderPipeline, RenderQueue, RenderStage, SceneCulling, Shadow, States
- PipelineSceneData: AmbientInfo, FogInfo, SkyboxInfo

#### Renderer/pipeline/custom
- RenderGraph (DAG-based pass/resource graph, Graphviz export)
- ResourceGraph (Managed/Persistent resource tracking, memory aliasing)
- LayoutGraph (DescriptorSetLayout/PipelineLayout management)
- FrameGraphDispatcher (three-phase compilation)
- NativePipeline

#### Renderer/pipeline/deferred
- GbufferStage (3 RGBA16F textures + depth)
- LightingStage (Tiled/Clustered/ForwardPlus modes)
- BloomStage (prefilter → downsample → upsample → combine)
- PostProcessStage (ToneMapping, GammaCorrection, FXAA, ColorGrading, Vignette)
- DeferredPipeline (full orchestration)

#### Renderer/core
- Material, Pass, Program, ProgramLib, Effect

#### 2D renderer
- Label, Sprite
- Batcher2D, RenderEntity, StencilManager, UIMeshBuffer, UIModelProxy

#### 3D renderer
- Mesh3D, Model, SkeletalAnimation, BakedSkinning, Morph models

#### Audio
- AudioDecoder (WAV/MP3/OGG), AudioUtils, AudioClip, AudioSource, AudioPlayer, AudioManager

#### UI
- Button, Layout, ScrollView, Widget, Toggle, ProgressBar
- VideoPlayer, WebView, EditBox, GridFlowLayout

#### Physics 3D
- PhysX backend framework, RigidBody, Joint, Shape, World, CharacterController, Simulator

#### Physics 2D
- PhysicsWorld2D, BuiltinWorld2D
- Intersection tests: AABB, Circle-Circle, AABB-Circle, Ray-Segment
- RigidBody2D, Collider2D, Joint2D

#### DragonBones
- Armature, Bone, Slot, Animation, Parser

#### Spine
- Skeleton, Bone, Animation, Parser

#### JS Bindings
- WasmBindgen, Native macro exports

#### Input, Tween, Particle, XR, etc.
- Input system, Tween system, 2D/3D particle systems, XR session management

#### Platform
- Linux window support, SystemWindow interfaces

#### Storage
- LocalStorage, JsonStorage, DiskStorage

#### Network
- HTTP, WebSocket, URI, HttpClient, Downloader, AsyncRuntime

#### Terrain
- HeightField (bilinear sampling), LODManager (4 levels), Terrain, TerrainAsset, TerrainBuffer

#### Tiled Map
- TMX parser, TileLayer (animation support), TiledMapAsset, TileMapOrientation

#### Sorting
- Sorting, SortingLayers

#### Others
- GI (LightProbe, AutoPlacement, Delaunay), Profiler (Counter, PerfCounter), Serialization (Serializer, Deserializer), Application (Root)

### Test Coverage

```
1700 tests passing, 0 failing
```

## Architecture Notes

- Column-major matrix layout matching OpenGL/GL conventions
- Rust's Copy trait for value types (no manual clone needed)
- Operator overloading via std::ops traits
- Zero-based indexing with const arrays
- No unsafe code in core modules

## Building

### Cargo (default)

```bash
cd cocos4-rust
cargo build --release
cargo test
```

### Bazel + BuildBuddy

The project is fully buildable with [Bazel](https://bazel.build) 8.x using
[`rules_rust`](https://github.com/bazelbuild/rules_rust) + `crate_universe`.
`Cargo.toml` remains the single source of truth for external dependencies and
features — `crate_universe` consumes `Cargo.lock` and emits Bazel targets for
every external crate at `@crates//:<name>`.

Remote cache and BES results are pushed to **BuildBuddy** (slug
`EGDoGaZis5ZMKN4P4TtB`).

Prerequisites:
- Bazelisk (recommended) or Bazel ≥ 8.0
- A C/C++ toolchain (for `cc`-dependent crates)
- `BUILDBUDDY_API_KEY` exported in your shell for remote-cache access
  (optional for local-only builds)

Common commands:

```bash
# Build everything in debug mode
bazel build //...

# Run all unit + integration tests
bazel test //:all_tests --test_output=errors

# Release build of the demo binary
bazel build //:cocos4-rust -c opt

# Run the demo binary
bazel run //:cocos4-rust

# Run the stand-alone quaternion demo
bazel run //:quaternion_demo

# With remote cache + BES (requires BUILDBUDDY_API_KEY)
bazel test //... --config=ci
```

CI: see `.github/workflows/ci.yml` — the `bazel` job runs against BuildBuddy
and prints an invocation link in the GitHub Actions step summary.

## License

MIT License - see LICENSE file for details.
