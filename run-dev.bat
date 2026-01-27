@echo off
echo Starting databases...
cd docker
docker-compose up -d
cd ..
echo.
echo Databases started!
echo.
echo Now open 2 more terminals and run these commands:
echo.
echo Terminal 2:
echo   cd C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine
echo   cargo run
echo.
echo Terminal 3:
echo   cd C:\Users\frank\Documents\scripts\trading-scripts\ladybug\gui
echo   npm run dev
echo.
echo Access the dashboard at: http://localhost:3000
echo.
pause