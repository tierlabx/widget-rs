# Pet Plugin 后续开发任务计划 (TODO)

## 已完成阶段 (Phase 1 & 2)
- [x] 基础框架：实现透明窗口、WGPU 渲染管线、GLTF 渲染。
- [x] 物理与拖拽：实现桌面上的自由拖拽、惯性与弹性效果。
- [x] 选项菜单：通过右键呼出菜单（已修复拖拽冲突与弹窗死锁）。
- [x] 核心功能：FPS设置 (60fps/30fps 切换)。
- [x] 模型管理：支持加载并渲染自定义 `.vrm` 模型，以及解除默认的 T-Pose 姿势。

---

## 阶段 3：动画系统与互动增强 (Phase 3: Animations & Interactions)
- [ ] **待机动画 (Idle Animation)**：
  - 在桌面上循环播放自然平滑的呼吸、轻微晃动。
  - 更流畅的动画过渡 (Blend Trees / Interpolation)。
- [ ] **拖拽动画 (Drag / Float)**：
  - 被移动或拖拽时，身体会轻轻漂浮和摆动（结合 Spine 惯性物理算法）。
- [ ] **触摸反应 (Touch Reactions)**：
  - 精确拾取射线 (Raycast) 检测，对脸部和头部的鼠标抚摸做出对应的动作或表情反应。

## 阶段 4：高级追踪与智能吸附 (Phase 4: Tracking & Placement)
- [ ] **追踪系统**：
  - **头部追踪 (Head Tracking)**：根据鼠标指针的位置，头和脖子自然转动看向鼠标。
  - **眼部追踪 (Eye Tracking)**：眼球跟随鼠标或随机眨眼。
  - **脊柱追踪 (Spine Tracking)**：身体根据视角轻微倾斜。
  - **手部动作 (Hand Gestures)**：根据互动情况做出简单手势。
- [ ] **智能坐立**：
  - **窗口坐立 (Window Sitting)**：检测到其它应用的窗口边缘，并坐在窗口上方。
  - **任务栏坐立 (Taskbar Sitting)**：识别任务栏的高度并在上面行走或坐下。

## 阶段 5：实验性功能与拓展 (Phase 5: Experimental & Mods)
- [ ] **音乐舞蹈 (Dancing)**：
  - **音乐反应**：捕获系统音频 (如 Spotify, Firefox 等) 的音量与频谱，跟随音乐节拍抖动或跳舞。
- [ ] **视觉特效**：
  - **粒子效果 (Particle Effects)**：点击、抚摸或跳舞时触发爱心、星星等 2D/3D 粒子效果。
- [ ] **自定义模组支持 (Custom Mod Support)**：
  - 允许外部通过 JSON 或配置文件自由添加专属声音 (SFX)、自定义粒子，增强 Metaengine 级别的扩展性。
- [ ] **性能监控与优化**：
  - 监控内存占用（确保维持在较低状态），释放不再使用的 GPU 纹理和 Buffer。
