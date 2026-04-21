# @flo-cli

# FLO-cli (FlowKit Command Line Interface)

**FLO-cli** is a high-performance Rust-based orchestration engine designed to bridge the gap between structured research in Obsidian and cinematic video production via Google Flow/VEO.

Built as part of the **Purple Pill Project** at Howard University, this tool implements an "Afrocomputation Praxis"—automating the technical labor of video rendering while maintaining strict human-in-the-loop safety guardrails for culturally aligned AI content.

## 🧬 Core Architecture

- **Obsidian Manifest Parser (`obsidian.rs`):** Ingests complex Markdown tables and frontmatter directly from your research vault.
- **Directorial Intent Injection:** Automatically prepends technical gear specs (e.g., Anamorphic 35mm f/1.4) and global aesthetics (e.g., 70s Cinema Prime) into every render request.
- **Human-in-the-Loop Gateway (`prompts.rs`):** A mandatory terminal-based pre-flight checklist that requires explicit researcher authorization (Y/N) before triggering API costs.
- **Asynchronous Polling (`status.rs`):** Utilizes `tokio` and `indicatif` for real-time progress tracking of cloud-based render workers.
- **Vault-Native Delivery (`download.rs`):** Automatically routes completed `.mp4` assets to specific project directories within your Obsidian vault.

## 🚀 Getting Started

### Prerequisites
- **Rust**: Ensure you have the Rust toolchain installed (stable-aarch64-apple-darwin recommended for macOS).
- **FlowKit API Server**: A running instance of the [FlowKit API Server](https://github.com/crisng95/flowkit).
- **Obsidian**: A vault containing your research manifests.

### Installation
```bash
git clone https://github.com/thanedouglass/flo-cli.git
cd flo-cli
cargo build --release
```

### Environment Configuration
Create a `.env` file in the root directory:
```env
FLOW_API_BASE_URL=http://127.0.0.1:8100
BEARER_TOKEN=your_token_here
```

## 🎬 Usage

To render a cinematic pilot from an Obsidian manifest:

```bash
./target/release/flo-cli render [MANIFEST_PATH] [CHARACTER_PATH]
```

**Example:**
```bash
./target/release/flo-cli render \
  ~/Desktop/vault/03-EPISODES/S01E01-LOUD-NOISE.md \
  ~/Desktop/vault/01-UNIVERSE/CHARACTERS/MATEO.md
```

## 🛡️ AI Safety & Praxis
This tool is built with **Algorithmic Accountability** at its core. By requiring a manual confirmation gate and utilizing local-first Markdown as the source of truth, FLO-cli ensures that the "Director" maintains full agency over the generative output, preventing automated "hallucinations" from entering the final production pipeline.

---
*Developed by Thane Douglas | Computer Science @ Howard University*
