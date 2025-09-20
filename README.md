# 🌾 Farm Sim Paradise

An interactive **Farm Simulation Game** built with **Rust + WebAssembly**.  
Players can grow, harvest, fertilize, fight pests, complete tasks, and manage resources through the in-game shop — all inside the browser.

---

## ✨ Features

- **Crop Lifecycle Management**  
  From planting → growing → harvesting → pest infestation, powered by a Rust state machine.
- **Shop & Inventory System**  
  Buy and use seeds, fertilizers, pesticides, insect nets, and manage coins strategically.
- **Task System**  
  Dynamic task tracking and reward collection to guide long-term play.
- **Pest System**  
  Random pest outbreaks requiring pesticide use for recovery, adding challenge.
- **Auto-Save Mechanism**  
  Game state is automatically saved in `localStorage`, enabling seamless resume.
- **Sound & Visual Feedback**  
  Each action comes with dedicated sound effects and tooltip feedback.
- **Modular Rust Architecture**  
  Core logic is implemented in Rust and connected to the frontend via wasm-bindgen.

---

## 🛠 Tech Stack

- **Language**: Rust  
- **Runtime**: WebAssembly  
- **Build Tool**: Trunk  
- **Frontend**: HTML5 + JavaScript + Canvas2D  
- **Audio**: `web_sys::HtmlAudioElement`  
- **Storage**: Browser `localStorage`  

---

## 📂 Project Structure

```
src/
 ├─ lib.rs        # Main wasm interface, game controller
 ├─ farm.rs       # Farm grid management & logic
 ├─ tile.rs       # TileState FSM (Empty/Planted/Mature/Infested)
 ├─ inventory.rs  # Inventory & item logic
 ├─ shop.rs       # Shop & economy system
 ├─ utils.rs      # Utility functions (sound, tooltip, logging)
```

---

## 🚀 Getting Started

### 1. Clone the repo
```bash
git clone https://github.com/your-username/farm-sim-paradise.git
cd farm-sim-paradise
```

### 2. Install dependencies
Make sure you have installed:
- [Rust](https://www.rust-lang.org/)  
- [Trunk](https://trunkrs.dev/)  

Install Trunk:
```bash
cargo install trunk
```

### 3. Run the development server
```bash
trunk serve
```

Open [http://localhost:8080](http://localhost:8080) in your browser 🎮.

---

## 🧪 Testing

- Functional test coverage includes:  
  - Crop lifecycle (plant/grow/harvest/pest)  
  - Shop & item system  
  - Task progression & rewards  
  - Save/load mechanism  
- Result: All modules work as expected, stable performance.  

---

## 📊 Highlights

- Compared to typical Wasm demos:  
  - Richer state machine (with pest system & insect nets)  
  - Enhanced interactions (drag-and-drop planting + tooltips)  
  - Long-term play supported (auto-save system)  
  - Immersive gameplay (sound effects, BGM, dynamic UI panels)

---

## 👥 Contributors

- **Zhou Yi** — Tile logic, state machine, music system, pest system, UI interactions  
- **Li Fengxing** — Drag & drop interactions, shop & inventory, save/load system  
- **Wen Jingwen** — Frontend rendering, fertilizing, crop progress tracking  
- **Liu Minghui** — Task system, experiment report writing  

---

## 📜 License

This project is for **educational and experimental purposes only**.  
