# 🐾 像素桌宠开发计划

> **项目代号**：Desktop Pet  
> **技术栈**：Rust + macroquad  
> **目标**：1-2 天内完成一个可运行的像素风桌宠 MVP  
> **最后更新**：2026-06-19

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术选型](#2-技术选型)
3. [架构设计](#3-架构设计)
4. [核心模块详解](#4-核心模块详解)
5. [像素美术规范](#5-像素美术规范)
6. [分阶段实施计划](#6-分阶段实施计划)
7. [状态机设计](#7-状态机设计)
8. [交互系统设计](#8-交互系统设计)
9. [好感度系统](#9-好感度系统)
10. [音效方案](#10-音效方案)
11. [项目结构](#11-项目结构)
12. [编译与运行](#12-编译与运行)
13. [后续扩展方向](#13-后续扩展方向)
14. [参考资料](#14-参考资料)

---

## 1. 项目概述

### 1.1 产品定义

一个运行在 Windows 桌面上的像素风小宠物，以透明窗口的形式存在，能够在屏幕底部自由漫步、与用户互动。宠物拥有简单的 AI 行为系统，会根据心情和状态做出不同的动作。

### 1.2 核心体验目标

- **可爱**：像素风角色 + 流畅动画，让人忍不住看一眼
- **轻量**：内存占用 < 30MB，CPU < 1%，不影响正常工作
- **互动**：右键喂食、鼠标靠近有反应、好感度积累
- **自运行**：不需要用户一直操作，宠物自己"活着"

### 1.3 MVP 功能清单（P0 = 必须，P1 = 尽量，P2 = 后续）

| 优先级 | 功能 | 说明 |
|--------|------|------|
| P0 | 透明窗口 | 宠物显示在桌面上，背景完全透明 |
| P0 | 行走动画 | 在屏幕底部左右漫步 |
| P0 | 边界检测 | 到达屏幕边缘自动转向 |
| P0 | 状态机 | 漫步 / 站立 / 睡觉 循环切换 |
| P1 | 吃东西 | 右键弹出菜单，喂食后播放吃东西动画 |
| P1 | 鼠标跟随 | 鼠标靠近时头转向鼠标方向 |
| P1 | 好感度 | 喂食增加好感，影响行为频率 |
| P1 | 发呆气泡 | 随机弹出想法气泡 |
| P2 | 拖拽移动 | 可以用鼠标拖拽宠物到新位置 |
| P2 | 多皮肤 | 切换宠物外观 |
| P2 | 系统托盘 | 最小化到托盘，支持退出 |
| P2 | 开机自启 | 注册 Windows 开机启动项 |

---

## 2. 技术选型

### 2.1 为什么选 Rust + macroquad

| 维度 | Rust + macroquad | C++ + SDL2 | 备注 |
|------|-----------------|------------|------|
| 开发速度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | macroquad API 极简 |
| 透明窗口 | ✅ 支持 | ✅ 支持 | 两者都行 |
| 跨平台 | ✅ Win/Mac/Linux | ✅ Win/Mac/Linux | |
| 像素渲染 | ✅ 原生支持 | ✅ 需手动设置 | macroquad 有 `TextureFilter::Nearest` |
| 编译产物 | 单个 .exe | 需附带 DLL | Rust 更方便分发 |
| 依赖管理 | Cargo（一键） | CMake/vcpkg（较繁琐） | |
| 学习曲线 | 中等 | 低 | |

**结论**：对于 1-2 天的 MVP，macroquad 的简洁 API 能极大加速开发。

### 2.2 核心依赖

```toml
[dependencies]
macroquad = "0.4"          # 游戏框架（窗口 + 渲染 + 输入）
macroquad-tiled = "0.2"    # 瓦片地图支持（可选，用于场景）
rand = "0.8"               # 随机数（状态切换时机）
```

### 2.3 透明窗口实现方案

macroquad 本身不直接支持透明窗口，需要借助平台特定 API：

**方案 A（推荐）：Win32 API 层级透明**
```rust
// 通过 windows crate 调用 Win32 API
// 设置窗口样式为 WS_EX_LAYERED + WS_EX_TRANSPARENT
// 使用 SetLayeredWindowAttributes 设置颜色键透明
```

**方案 B：自制窗口 + winit + softbuffer**
- 完全控制窗口行为，但代码量更大
- 适合需要更精细控制的场景

**MVP 阶段建议**：先用方案 A，快速验证效果。

---

## 3. 架构设计

### 3.1 整体架构图

```
┌──────────────────────────────────────────┐
│              main.rs (主循环)              │
│  ┌─────────────────────────────────────┐ │
│  │           Game Loop (60 FPS)        │ │
│  │  ┌──────────┐  ┌────────────────┐  │ │
│  │  │  Input   │  │    Update      │  │ │
│  │  │  Layer   │──│  ┌──────────┐  │  │ │
│  │  │(鼠标/键盘)│  │  │Pet State │  │  │ │
│  │  └──────────┘  │  │ Machine  │  │  │ │
│  │                │  └──────────┘  │  │ │
│  │                │  ┌──────────┐  │  │ │
│  │                │  │Animation │  │  │ │
│  │                │  │ Player   │  │  │ │
│  │                │  └──────────┘  │  │ │
│  │                └────────────────┘  │ │
│  │  ┌──────────────────────────────┐  │ │
│  │  │         Render Layer         │  │ │
│  │  │  精灵绘制 / 气泡 / 粒子效果   │  │ │
│  │  └──────────────────────────────┘  │ │
│  └─────────────────────────────────────┘ │
└──────────────────────────────────────────┘
         │                    ▲
         ▼                    │
┌──────────────┐    ┌─────────────────┐
│  assets/     │    │   Win32 API     │
│  精灵表/音效  │    │  透明窗口管理    │
└──────────────┘    └─────────────────┘
```

### 3.2 设计原则

1. **单一职责**：每个模块只管一件事
2. **状态驱动**：宠物行为完全由状态机控制
3. **数据驱动**：动画帧、行为参数都用数据定义，不硬编码
4. **可测试**：核心逻辑（状态机、好感度）不依赖渲染，可独立测试

---

## 4. 核心模块详解

### 4.1 `pet.rs` — 宠物实体

```rust
pub struct Pet {
    // 位置与物理
    pub x: f32,
    pub y: f32,
    pub velocity_x: f32,
    pub direction: Direction,       // Left / Right

    // 状态
    pub state: PetState,
    pub state_timer: f32,           // 当前状态已持续时间
    pub mood: Mood,                 // 开心 / 普通 / 不开心

    // 好感度
    pub affection: f32,             // 0.0 ~ 100.0

    // 动画
    pub animation: AnimationPlayer,

    // 气泡
    pub bubble: Option<SpeechBubble>,
}
```

### 4.2 `animation.rs` — 动画播放器

```rust
pub struct AnimationPlayer {
    pub spritesheet: Texture2D,     // 精灵表纹理
    pub frame_width: f32,           // 单帧宽度
    pub frame_height: f32,          // 单帧高度
    pub current_animation: String,  // 当前动画名称
    pub current_frame: usize,       // 当前帧索引
    pub frame_timer: f32,           // 帧计时器
    pub frame_duration: f32,        // 每帧持续时间（秒）
    pub animations: HashMap<String, AnimationDef>,
}

pub struct AnimationDef {
    pub frames: Vec<(usize, usize)>,  // (col, row) 在精灵表中的位置
    pub loop_animation: bool,
    pub speed: f32,                    // 帧率
}

impl AnimationPlayer {
    pub fn update(&mut self, dt: f32) { /* 推进帧 */ }
    pub fn play(&mut self, name: &str) { /* 切换动画 */ }
    pub fn draw(&self, x: f32, y: f32, flip: bool) { /* 绘制当前帧 */ }
}
```

### 4.3 `interaction.rs` — 交互管理

```rust
pub struct InteractionManager {
    pub right_click_menu: Option<ContextMenu>,
    pub is_dragging: bool,
    pub drag_offset: (f32, f32),
}

pub enum MenuItem {
    Feed,           // 喂食
    Pet,            // 摸摸头
    ToggleSound,    // 静音/开音效
    Quit,           // 退出
}
```

---

## 5. 像素美术规范

### 5.1 角色设计

| 属性 | 值 |
|------|-----|
| 画布尺寸 | 32×32 像素（放大 2-3 倍显示） |
| 色板 | 限制 8-16 色，增强像素风辨识度 |
| 风格 | 圆润可爱，大眼睛，小短腿 |
| 推荐配色 | 暖色调为主（奶油色身体 + 橙色点缀） |

### 5.2 动画帧规划

每组动画做成精灵表的一行：

```
精灵表布局 (每行一组动画):
Row 0: 站立发呆 (2帧, 慢速, 循环)
Row 1: 向右行走 (4帧, 中速, 循环)
Row 2: 睡觉     (2帧, 慢速, 循环, 含 zzz 效果)
Row 3: 吃东西   (3帧, 快速, 播放一次)
Row 4: 开心     (2帧, 中速, 播放一次, 爱心效果)
Row 5: 受惊     (2帧, 快速, 播放一次)
Row 6: 坐下     (2帧, 慢速, 循环)
```

### 5.3 UI 元素

| 元素 | 尺寸 | 说明 |
|------|------|------|
| 对话气泡 | 48×24 px | 圆角矩形 + 小尾巴 |
| 思考气泡 | 24×24 px | 三个递增圆点 |
| 爱心粒子 | 8×8 px | 飘出后淡出 |
| zzz 字符 | 16×16 px | 从头顶飘出 |
| 右键菜单 | 80×100 px | 像素风菜单框 |

### 5.4 美术工具推荐

| 工具 | 价格 | 推荐度 | 说明 |
|------|------|--------|------|
| **Aseprite** | $19.99 | ⭐⭐⭐⭐⭐ | 像素画的行业标准，支持动画预览 |
| **Piskel** | 免费 | ⭐⭐⭐⭐ | 在线工具，轻量够用 |
| **LibreSprite** | 免费 | ⭐⭐⭐ | Aseprite 的免费开源分支 |
| **GraphicsGale** | 免费 | ⭐⭐⭐ | 老牌像素画工具 |

### 5.5 占位美术（开发阶段）

在最终美术资源就绪前，使用色块精灵表快速开发：

```
站立：绿色方块 (32x32)
行走：绿色方块 + 蓝色小脚 (交替)
睡觉：绿色方块 + "zzz" 文字
吃东西：绿色方块 + 橙色小方块
```

可以先用程序生成占位精灵表，保证开发不被美术阻塞。

---

## 6. 分阶段实施计划

### Phase 1：窗口与渲染基础（2-3 小时）

**目标**：在桌面上显示一个半透明的像素方块

```
里程碑：一个绿色方块出现在屏幕右下角
```

**任务清单**：

- [ ] 初始化 Rust 项目 (`cargo init desktop-pet`)
- [ ] 配置 `Cargo.toml` 依赖
- [ ] 创建 macroquad 窗口（固定尺寸 200×200）
- [ ] 实现 Win32 透明窗口（关键步骤）：
  - 获取窗口句柄 (HWND)
  - 设置 `WS_EX_LAYERED` 扩展样式
  - 使用 `SetLayeredWindowAttributes` 设置品红色 (255, 0, 255) 为透明键
  - 设置窗口置顶 (`WS_EX_TOPMOST`)
  - 去掉窗口标题栏
- [ ] 加载并绘制一个占位精灵
- [ ] 验证精灵在桌面上正确显示

**关键代码参考**：

```rust
// Win32 透明窗口设置（伪代码）
unsafe {
    let hwnd = get_window_handle();
    
    // 获取当前样式
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    
    // 添加分层窗口和透明样式
    SetWindowLongW(hwnd, GWL_EXSTYLE, 
        ex_style | WS_EX_LAYERED as i32 | WS_EX_TRANSPARENT as i32);
    
    // 设置品红色为透明色键
    SetLayeredWindowAttributes(
        hwnd, 
        RGB(255, 0, 255),  // 透明色
        255,               // 不透明度
        LWA_COLORKEY       // 使用色键模式
    );
    
    // 窗口置顶
    SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, 
        SWP_NOMOVE | SWP_NOSIZE);
}
```

**踩坑预警**：
- macroquad 默认会在背景清除为黑色，需要改为品红色 (magenta) 作为透明色键
- 窗口大小要足够容纳宠物 + 气泡 + 特效，建议 200×200
- 部分 Windows 版本需要关闭"窗口动画"才能正常显示

---

### Phase 2：动画系统（1-2 小时）

**目标**：精灵表加载 + 帧动画播放

```
里程碑：方块变成了会动的角色
```

**任务清单**：

- [ ] 创建占位精灵表 (可用代码生成，或用 Piskel 画)
- [ ] 实现 `AnimationPlayer` 结构体
- [ ] 实现精灵表切割逻辑（按行按列读取帧）
- [ ] 实现帧计时和自动推进
- [ ] 实现动画切换 (`play("walk")` → `play("idle")`)
- [ ] 实现水平翻转（角色朝左/朝右）
- [ ] 测试：角色能在站立和行走之间切换

**精灵表加载示例**：

```rust
let texture = load_texture("assets/pet.png").await.unwrap();
// 设置为最近邻采样，保证像素清晰
texture.set_filter(FilterMode::Nearest);

let anim = AnimationPlayer::new(texture, 32.0, 32.0);
anim.define("idle", vec![(0,0), (1,0)], 2.0, true);
anim.define("walk", vec![(0,1), (1,1), (2,1), (3,1)], 8.0, true);
anim.define("sleep", vec![(0,2), (1,2)], 1.0, true);
```

---

### Phase 3：状态机与行为逻辑（2-3 小时）

**目标**：宠物能自主"生活"——走路、站立、睡觉循环

```
里程碑：宠物自己在屏幕底部走来走去，偶尔停下来
```

**任务清单**：

- [ ] 实现 `PetState` 枚举和状态机转换逻辑
- [ ] 实现 **漫步状态**：匀速移动 + 边界转向
- [ ] 实现 **站立状态**：随机持续 2-5 秒
- [ ] 实现 **睡觉状态**：原地不动 5-10 秒
- [ ] 实现状态切换的随机权重（站立 30%，漫步 50%，睡觉 20%）
- [ ] 将宠物 Y 坐标固定在屏幕底部
- [ ] 实现 `update(dt)` 主更新函数
- [ ] 测试：运行 5 分钟，观察行为是否自然

**状态转换表**：

```
当前状态  │ 可能的下一个状态          │ 触发条件
─────────┼─────────────────────────┼──────────────
站立     │ 漫步(60%), 睡觉(30%), 站立(10%) │ timer > rand(2~5s)
漫步     │ 站立(50%), 吃东西(30%), 漫步(20%) │ timer > rand(3~8s)
睡觉     │ 站立(80%), 漫步(20%)      │ timer > rand(5~15s)
吃东西   │ 站立(70%), 开心(30%)      │ 动画播放完毕
```

---

### Phase 4：交互系统（1-2 小时）

**目标**：右键喂食、鼠标靠近有反应

```
里程碑：能和宠物互动了！
```

**任务清单**：

- [ ] 实现鼠标位置检测（相对于窗口）
- [ ] 实现**鼠标悬停反应**：鼠标靠近时角色头转向鼠标
- [ ] 实现**右键菜单**：
  - 像素风菜单 UI
  - 菜单项：喂食 🍖 / 摸摸头 ✋ / 退出 ❌
  - 点击菜单外区域关闭
- [ ] 实现**喂食逻辑**：
  - 触发吃东西动画
  - 好感度 +5
  - 播放爱心粒子
- [ ] 实现**摸摸头逻辑**：
  - 触发开心动画
  - 好感度 +3
- [ ] 测试：所有交互流程正常

**右键菜单渲染**：

```rust
fn draw_context_menu(menu: &ContextMenu) {
    // 背景框：深棕色边框 + 浅棕色填充
    draw_rectangle(x, y, width, height, Color::new(0.2, 0.15, 0.1, 1.0));
    // 每个菜单项
    for (i, item) in menu.items.iter().enumerate() {
        let bg = if menu.hovered == i { light_color } else { transparent };
        draw_rectangle(x+2, y + i as f32 * 20.0 + 2.0, width-4, 18.0, bg);
        draw_text(&item.label, x + 8.0, y + i as f32 * 20.0 + 15.0, 16.0, WHITE);
    }
}
```

---

### Phase 5：好感度与心情系统（1 小时）

**目标**：宠物有"情感"，行为随好感度变化

```
里程碑：好感度影响宠物行为
```

**任务清单**：

- [ ] 实现 `AffectionSystem`：
  - 好感度范围 0-100
  - 每 30 秒自动 -1（保持互动的动力）
  - 喂食 +5，摸头 +3
- [ ] 实现心情影响行为：
  - 好感度 < 20：经常发呆、不开心动画、远离鼠标
  - 好感度 20-60：正常行为
  - 好感度 > 60：经常开心、主动靠近鼠标、掉落爱心
- [ ] 实现心情指示器（可选：头顶小表情图标）
- [ ] 测试：观察不同好感度下行为差异

**心情映射表**：

```
好感度区间 │ 心情   │ 行为特征
──────────┼────────┼──────────────────────────
  0 ~ 20  │ 😢 不开心 │ 走路慢, 站立多, 头低下, 离鼠标远
 20 ~ 40  │ 😐 普通   │ 正常行为频率
 40 ~ 60  │ 🙂 不错   │ 偶尔开心动画, 走路轻快
 60 ~ 80  │ 😊 开心   │ 频繁开心, 跳跃行走, 靠近鼠标
 80 ~ 100 │ 🥰 非常开心 │ 主动跟随鼠标, 常掉爱心, 跳舞
```

---

### Phase 6：打磨与发布（1-2 小时）

**目标**：从"能跑"变成"能用"

```
里程碑：可以分享给朋友了
```

**任务清单**：

- [ ] 替换占位美术为正式像素美术（或优化占位版本）
- [ ] 添加窗口拖拽功能（左键按住拖拽移动宠物位置）
- [ ] 添加系统托盘图标 + 右键退出菜单
- [ ] 调整动画细节（帧率、过渡、抖动等）
- [ ] 添加简单音效（可选，需要 `macroquad` 音频模块）
- [ ] Release 模式编译优化 (`cargo build --release`)
- [ ] 测试最终产物：
  - 不同分辨率下的表现
  - 长时间运行稳定性（跑 1 小时）
  - 内存/CPU 占用检查
- [ ] 打包发布：单个 .exe + assets 文件夹

---

## 7. 状态机设计

### 7.1 状态定义

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PetState {
    Idle,           // 站立发呆
    Walking,        // 漫步
    Sleeping,       // 睡觉
    Eating,         // 吃东西
    Happy,          // 开心
    Startled,       // 受惊（鼠标快速移动触发）
    Sitting,        // 坐下
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
}
```

### 7.2 状态转换流程图

```
                    ┌─────────┐
            ┌──────│  Idle   │──────┐
            │      └────┬────┘      │
            │           │           │
            ▼           ▼           ▼
     ┌──────────┐ ┌──────────┐ ┌──────────┐
     │ Walking  │ │ Sleeping │ │  Sitting  │
     └────┬─────┘ └──────────┘ └──────────┘
          │
          │ (右键喂食)
          ▼
     ┌──────────┐
     │  Eating  │─────┐
     └────┬─────┘     │
          │           │
          ▼           ▼
     ┌──────────┐  (好感度>60)
     │  Happy   │  30%概率触发
     └──────────┘

     (鼠标快速移动靠近)
     ┌───────────┐
     │ Startled  │ → 自动回 Idle (1s后)
     └───────────┘
```

### 7.3 主更新循环伪代码

```rust
fn update(pet: &mut Pet, dt: f32, mouse_pos: Vec2) {
    // 1. 更新状态计时器
    pet.state_timer += dt;

    // 2. 检查是否需要切换状态
    if pet.should_transition() {
        let next = pet.decide_next_state();
        pet.transition_to(next);
    }

    // 3. 执行当前状态行为
    match pet.state {
        PetState::Idle => { /* 微小摇晃动画 */ }
        PetState::Walking => {
            pet.x += pet.velocity_x * dt;
            // 边界检测
            if pet.x < 0.0 || pet.x > screen_width() - 32.0 {
                pet.direction = pet.direction.flip();
                pet.velocity_x = -pet.velocity_x;
            }
        }
        PetState::Sleeping => { /* zzz 飘出 */ }
        PetState::Eating => { /* 播放完毕后回到 Idle */ }
        PetState::Happy => { /* 掉落爱心粒子 */ }
        _ => {}
    }

    // 4. 更新动画
    pet.animation.update(dt);

    // 5. 更新粒子效果
    pet.particles.update(dt);

    // 6. 检查鼠标交互
    if is_mouse_near(pet, mouse_pos) {
        pet.look_at(mouse_pos);
    }
}
```

---

## 8. 交互系统设计

### 8.1 输入处理

```rust
fn handle_input(pet: &mut Pet) {
    let mouse = mouse_position();
    
    // 鼠标悬停检测
    let distance = distance(mouse, pet.center());
    if distance < 50.0 {
        pet.on_mouse_approach(mouse);
    }

    // 右键点击 → 弹出菜单
    if is_mouse_button_pressed(MouseButton::Right) {
        if pet.hit_test(mouse) {
            pet.show_context_menu(mouse);
        }
    }

    // 左键拖拽
    if is_mouse_button_down(MouseButton::Left) {
        if pet.is_dragging {
            pet.x = mouse.0 - pet.drag_offset.0;
            pet.y = mouse.1 - pet.drag_offset.1;
        } else if pet.hit_test(mouse) {
            pet.is_dragging = true;
            pet.drag_offset = (mouse.0 - pet.x, mouse.1 - pet.y);
            pet.transition_to(PetState::Startled);
        }
    } else {
        pet.is_dragging = false;
    }
}
```

### 8.2 右键菜单设计

```
┌──────────────┐
│ 🍖 喂食       │  → 播放 Eating 动画, 好感度+5
│ ✋ 摸摸头      │  → 播放 Happy 动画, 好感度+3
│ 💤 去睡觉     │  → 强制进入 Sleeping
│─────────────│
│ ❌ 退出       │  → 关闭程序
└──────────────┘
```

菜单出现位置：鼠标右键点击处，向上偏移避免超出屏幕。

---

## 9. 好感度系统

### 9.1 数值设计

```rust
pub struct AffectionSystem {
    pub value: f32,           // 0.0 ~ 100.0
    pub decay_rate: f32,      // 自然衰减速率：每30秒 -1.0
    pub decay_timer: f32,     // 衰减计时器
}

impl AffectionSystem {
    pub fn feed(&mut self) {
        self.value = (self.value + 5.0).min(100.0);
    }

    pub fn pet_head(&mut self) {
        self.value = (self.value + 3.0).min(100.0);
    }

    pub fn tick(&mut self, dt: f32) {
        self.decay_timer += dt;
        if self.decay_timer >= 30.0 {
            self.value = (self.value - 1.0).max(0.0);
            self.decay_timer = 0.0;
        }
    }

    pub fn mood(&self) -> Mood {
        match self.value as u32 {
            0..=20  => Mood::Unhappy,
            21..=40 => Mood::Neutral,
            41..=60 => Mood::Content,
            61..=80 => Mood::Happy,
            _       => Mood::Ecstatic,
        }
    }
}
```

### 9.2 好感度对行为的影响

| 行为参数 | 低好感 (0-20) | 中好感 (40-60) | 高好感 (80-100) |
|----------|---------------|----------------|-----------------|
| 行走速度 | 慢 (30px/s) | 正常 (60px/s) | 快 (80px/s) |
| 睡觉频率 | 高 (40%) | 正常 (20%) | 低 (10%) |
| 站立时长 | 长 (5-10s) | 正常 (2-5s) | 短 (1-2s) |
| 鼠标反应 | 逃跑 | 好奇看 | 靠近 |
| 随机开心 | 0% | 10% | 40% |

---

## 10. 音效方案

### 10.1 音效清单（可选，Phase 6 添加）

| 事件 | 音效描述 | 建议来源 |
|------|----------|----------|
| 吃东西 | 咔嚓咔嚓 | freesound.org |
| 好感度增加 | 叮~ 清脆提示音 | 自制 sine wave |
| 睡觉 | 轻微呼噜声 | 可选，或静音 |
| 受惊 | "嗯？" 短促惊讶 | 可选 |
| 右键菜单弹出 | 轻微 pop | 自制 |
| 心情变差 | 低沉的叹气 | 可选 |

### 10.2 实现

```rust
// macroquad 音频
use macroquad::audio::{load_sound, play_sound_once};

let eat_sound = load_sound("assets/eat.wav").await.unwrap();
play_sound_once(&eat_sound);
```

**注意**：音效是锦上添花，MVP 阶段完全可以跳过。

---

## 11. 项目结构

```
desktop-pet/
├── Cargo.toml                 # 项目配置与依赖
├── Cargo.lock                 # 依赖锁文件
├── README.md                  # 项目说明
├── assets/                    # 资源文件
│   ├── pet.png               # 角色精灵表 (32x32 per frame)
│   ├── bubble.png            # 气泡素材（可选）
│   ├── ui.png                # UI 元素素材（可选）
│   ├── eat.wav               # 吃东西音效（可选）
│   └── pop.wav               # 菜单弹出音效（可选）
├── src/
│   ├── main.rs               # 入口：窗口创建 + 主循环
│   ├── pet.rs                # Pet 结构体 + 状态机
│   ├── animation.rs          # 动画系统
│   ├── interaction.rs        # 输入处理 + 右键菜单
│   ├── affection.rs          # 好感度系统
│   ├── particles.rs          # 粒子效果（爱心、zzz）
│   ├── config.rs             # 常量配置（速度、颜色等）
│   └── win32.rs              # Win32 API 透明窗口封装
├── tools/                     # 开发辅助工具
│   └── gen_placeholder.py    # 生成占位精灵表的 Python 脚本
└── docs/
    └── plan.md               # 本文档
```

---

## 12. 编译与运行

### 12.1 环境准备

```bash
# 安装 Rust（如未安装）
# https://rustup.rs

# 创建项目
cargo init desktop-pet
cd desktop-pet

# 添加依赖（编辑 Cargo.toml）
# macroquad = "0.4"
# rand = "0.8"
# [target.'cfg(windows)'.dependencies]
# windows = { version = "0.56", features = ["Win32_Foundation", "Win32_Graphics_Gdi", "Win32_UI_WindowsAndMessaging"] }
```

### 12.2 编译运行

```bash
# 开发模式（快速编译，不优化）
cargo run

# Release 模式（优化，用于发布）
cargo run --release

# 仅编译
cargo build --release
# 输出：target/release/desktop-pet.exe
```

### 12.3 打包发布

```
发布文件清单：
├── desktop-pet.exe     # 可执行文件（约 5-10 MB）
└── assets/             # 资源文件夹
    └── pet.png
```

将这两项放在同一目录下，双击 exe 即可运行。

### 12.4 可选：嵌入资源

使用 `include_bytes!` 将资源嵌入二进制文件，实现单文件发布：

```rust
const PET_SPRITES: &[u8] = include_bytes!("../assets/pet.png");
// 从内存加载而非文件
let texture = Texture2D::from_file_with_format(PET_SPRITES, None);
```

---

## 13. 后续扩展方向

完成 MVP 后，可以考虑以下扩展（按趣味程度排序）：

### 13.1 高优先级扩展

| 扩展 | 工作量 | 说明 |
|------|--------|------|
| 🎨 多套皮肤 | 2-3h | 右键菜单切换不同角色外观 |
| 🎵 背景音乐 | 1-2h | 轻松的 chiptune 音乐循环播放 |
| 🖱️ 双屏支持 | 1h | 宠物可在多个显示器间走动 |
| 📌 开机自启 | 1h | 写注册表实现 |

### 13.2 中优先级扩展

| 扩展 | 工作量 | 说明 |
|------|--------|------|
| 🎮 小游戏 | 4-6h | 点击小游戏（打地鼠/接东西） |
| 📋 待办提醒 | 3-4h | 宠物提醒你喝水/休息/待办 |
| 🌤️ 天气显示 | 2-3h | 宠物旁边显示当前天气小图标 |
| 💾 存档系统 | 2h | 好感度和设置持久化到文件 |

### 13.3 低优先级（好玩但费时）

| 扩展 | 工作量 | 说明 |
|------|--------|------|
| 🤖 AI 对话 | 4-6h | 接入 LLM API，宠物能聊天 |
| 🌐 在线对战 | 8h+ | 两只宠物在桌面上互动 |
| 🔌 插件系统 | 6-8h | 用户可编写自定义行为脚本 |
| 📱 手机联动 | 8h+ | 手机通知同步到桌面宠物 |

---

## 14. 参考资料

### macroquad 相关
- [macroquad 官方文档](https://docs.rs/macroquad)
- [macroquad GitHub](https://github.com/not-fl3/macroquad)
- [macroquad 示例集](https://github.com/not-fl3/macroquad/tree/master/examples)

### 像素美术
- [Piskel 在线像素画编辑器](https://www.piskelapp.com)
- [Lospec 色板库](https://lospec.com/palette-list) — 挑选合适的像素风色板
- [Pixel Art 颜色理论](https://lospec.com/pixel-art-tutorials)

### Win32 API
- [SetLayeredWindowAttributes 文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes)
- [WS_EX_LAYERED 说明](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)

### 灵感参考
- [Shimeji-ee](https://shimeji-ee.github.io) — 经典桌面宠物
- [Desktop Goose](https://samperson.itch.io/desktop-goose) — 搞怪桌面鹅
- [Oneko](https://github.com/adryd325/oneko) — 简单的跟随鼠标小猫

---

## 附录 A：Cargo.toml 完整配置

```toml
[package]
name = "desktop-pet"
version = "0.1.0"
edition = "2021"
description = "A cute pixel art desktop pet"

[dependencies]
macroquad = "0.4"
rand = "0.8"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.56", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_UI_WindowsAndMessaging",
] }

[profile.release]
opt-level = 3
lto = true
strip = true
```

## 附录 B：快速占位精灵表生成

```python
#!/usr/bin/env python3
"""生成占位精灵表 - tools/gen_placeholder.py"""
from PIL import Image

W, H = 32, 32
COLS, ROWS = 4, 7  # 4帧宽, 7组动画
img = Image.new("RGBA", (W * COLS, H * ROWS), (255, 0, 255, 255))  # 品红背景(透明)

colors = [
    (100, 200, 100),  # 站立-绿
    (100, 200, 100),  # 行走-绿
    (100, 100, 200),  # 睡觉-蓝
    (200, 150, 100),  # 吃东西-橙
    (200, 100, 100),  # 开心-粉
    (200, 200, 100),  # 受惊-黄
    (100, 200, 200),  # 坐下-青
]

for row, color in enumerate(colors):
    for col in range(COLS):
        x0, y0 = col * W, row * H
        # 画一个简单的方块角色
        for px in range(8, 24):
            for py in range(8, 24):
                img.putpixel((x0 + px, y0 + py), (*color, 255))
        # 行走帧加个小脚偏移
        if row == 1:
            offset = (col % 2) * 2
            for px in range(10, 14):
                img.putpixel((x0 + px + offset, y0 + 24), (*color, 255))
            for px in range(18, 22):
                img.putpixel((x0 + px - offset, y0 + 24), (*color, 255))

img.save("assets/pet.png")
print("Generated assets/pet.png")
```

---

> **文档版本**：v1.0  
> **作者**：Desktop Pet Project  
> **状态**：计划阶段
