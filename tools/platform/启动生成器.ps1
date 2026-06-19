# 桌宠精灵表生成器启动脚本
Write-Host "正在启动桌宠精灵表生成器..." -ForegroundColor Cyan
$htmlPath = Join-Path $PSScriptRoot "index.html"
if (Test-Path $htmlPath) {
    Start-Process $htmlPath
    Write-Host "已在浏览器中打开生成器" -ForegroundColor Green
} else {
    Write-Host "错误：找不到 index.html 文件" -ForegroundColor Red
}
