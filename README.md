# Cocos4-Rust

Rust implementation of Cocos4 game engine.

## Progress

This is a work-in-progress conversion of Cocos4 engine from C++ to Rust.

### Completed Modules (28/48 - 58%)

#### Math module ✅
- Vec2, Vec3, Vec4, Mat3, Mat4, Quaternion, Color, Geometry

#### Base module ✅
- Types, RefCounted, Log, Scheduler, Timer, ObjectPool, Data, Value, Util
- **threading** (NEW): ReadWriteLock, MessageQueue, ThreadPool, ThreadSafeCounter, AutoReleasePool

#### Core module ✅
- SceneGraph (Node/Scene/Transform)
- Geometry: AABB, Sphere, OBB, Ray, Line, Plane, Frustum
  - **NEW**: Capsule, Triangle, Spline, AnimationCurve
- Animation, EventSystem, EventBus, EventTarget, StateMachine, SpatialGrid
- Assets: Material, Mesh, Texture, Font, Image, Effect
- **memop** (NEW): Pool, RecyclePool, CachedArray

#### Renderer/gfx-base ✅
- Format (complete, 130+ variants), BufferUsage, MemoryUsage, TextureType/Usage/Flags
- **NEW**: Feature, ObjectType, Status, FormatFeature, ColorMask, CullMode, PolygonMode
- **NEW**: DynamicStateFlags, StencilFace, AccessFlags, ClearFlags, BarrierType
- **NEW**: DrawInfo, DispatchInfo, BufferTextureCopy, TextureBlit, TextureCopy
- **NEW**: DeviceCaps, Offset, Extent, TextureSubresLayers, MarkerInfo
- Buffer, Texture, Shader, Sampler, RenderPass, Framebuffer
- **command_buffer** (ENHANCED): begin_render_pass, bind_pipeline, bind_descriptor_set, pipeline_barrier, blit_texture, copy_texture, dispatch, begin/end_query
- **device** (ENHANCED): create_queue, create_query_pool, create_swapchain, create_render_pass, create_framebuffer, create_input_assembler, create_descriptor_set, create_pipeline_layout, create_pipeline_state, flush_commands

#### Renderer/frame-graph ✅
- Blackboard, PassNode, RenderTargetAttachment
- **FrameGraph** (NEW): add_pass, create_texture, import_external_texture, compile, execute, reset
- **ResourceNode, VirtualResource** (NEW)

#### Renderer/pipeline ✅
- Defines, RenderFlow, RenderPipeline, RenderQueue, RenderStage, SceneCulling, Shadow, States
- **PipelineSceneData** (NEW): AmbientInfo, FogInfo, SkyboxInfo, render objects management
- **NEW**: RenderObject, RenderTextureDesc, FrameBufferDesc, RenderFlowType

#### Renderer/core ✅
- Material, Pass
- **ProgramLib** (NEW): define, get_template, has_program, destroy

#### 2D renderer ✅
- Label, Sprite
- Batcher2D (ENHANCED): update, upload_buffers, reset, fill_buffers_and_merge_batches, sync_mesh_buffers
- DrawInfo, RenderEntity, StencilManager, UIMeshBuffer (ENHANCED: upload method), UIModelProxy

#### 3D renderer ✅
- Mesh3D, Model, SkeletalAnimation

#### Audio ✅
- AudioDecoder (WAV/MP3/OGG), AudioUtils
- AudioClip, AudioSource, AudioPlayer, AudioManager
- **NEW**: INVALID_AUDIO_ID, TIME_UNKNOWN constants, AudioProfile

#### UI ✅
- Button, Layout, ScrollView, Widget, Toggle

#### Physics ✅
- PhysX backend framework

#### Input ✅
- Input system

#### Tween ✅
- Tween system

#### Particle ✅
- 2D/3D particle systems

#### XR ✅
- XR session management

### Test Coverage

```
1121 tests passing, 0 failing
```

## Architecture Notes

- Column-major matrix layout matching OpenGL/GL conventions
- Rust's Copy trait for value types (no manual clone needed)
- Operator overloading via std::ops traits
- Zero-based indexing with const arrays
- No unsafe code in core modules

## Building

```bash
cd cocos4-rust
cargo build --release
```

## Testing

```bash
cargo test
```

## License

MIT License - see LICENSE file for details.
