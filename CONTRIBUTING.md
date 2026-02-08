# 贡献指南 (Contributing Guide)

感谢你对 AppManager 的关注！我们非常欢迎各种形式的贡献，无论是报告 Bug、提出新功能建议，还是直接提交代码。

## 🛠️ 环境准备

在开始贡献之前，请确保你的开发环境已准备好：

- **Rust**: 最新稳定版。
- **Node.js**: v18+ 推荐。
- **pnpm**: `npm install -g pnpm`。
- **Windows 开发工具**: 由于本项目针对 Windows 平台，建议在 Windows 环境下开发。

## 📝 提交 Issue

如果你发现了 Bug 或者有新的想法：

1. 先在 Issue 列表中搜索，看是否已经有人提出过。
2. 如果没有，请创建一个新 Issue。
3. 清晰地描述问题或建议，如果可能的话，提供重现步骤。

## 💻 提交代码 (Pull Request)

1. **Fork** 本仓库到你自己的 GitHub 账号下。
2. **Clone** 仓库到本地。
3. **创建分支**: `git checkout -b feature/your-feature-name` 或 `git checkout -b fix/your-bug-name`。
4. **进行开发**: 请遵循项目现有的代码风格和架构模式。
5. **本地测试**: 运行 `pnpm tauri dev` 确保功能正常，且没有引入新的 Bug。
6. **提交代码**: `git commit -m 'feat: 描述你的新功能'`.
7. **推送分支**: `git push origin your-branch-name`.
8. **提交 PR**: 在 GitHub 上向主仓库发起 Pull Request。

## 📏 代码风格

- **Rust**: 使用 `cargo fmt` 进行代码格式化。
- **TypeScript/React**: 遵循项目中 ESLint 和 Prettier 的配置。
- **提交信息**: 推荐使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范。

## 💬 沟通交流

如果你有任何疑问，欢迎通过 Issue 与我们沟通。

再次感谢你的贡献！
