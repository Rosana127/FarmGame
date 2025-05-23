const express = require('express');
const path = require('path');
const app = express();

// 设置正确的 MIME 类型
app.use((req, res, next) => {
  if (req.url.endsWith('.js')) {
    res.type('application/javascript');
  } else if (req.url.endsWith('.wasm')) {
    res.type('application/wasm');
  }
  next();
});

// 提供静态文件
app.use(express.static(path.join(__dirname, 'dist')));

const PORT = 8080;
app.listen(PORT, () => {
  console.log(`服务器运行在 http://localhost:${PORT}`);
}); 