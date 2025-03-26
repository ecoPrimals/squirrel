# Simple script to build the UI assets by copying files from web to dist
# This is a temporary solution until we have a proper build system in place

# Create dist directory if it doesn't exist
New-Item -ItemType Directory -Force -Path "dist"

# Clear existing files in dist
Remove-Item -Path "dist\*" -Recurse -Force -ErrorAction SilentlyContinue

# Copy all files from web to dist
Copy-Item -Path "web\*" -Destination "dist" -Recurse -Force

# Update paths in the HTML file to point to the right locations
(Get-Content -Path "dist\index.html") -replace 'href="css/', 'href="' -replace 'src="js/', 'src="' | Set-Content -Path "dist\index.html"

# Copy CSS files to root of dist
Copy-Item -Path "dist\css\*" -Destination "dist" -Force

# Copy JS files to root of dist
Copy-Item -Path "dist\js\*" -Destination "dist" -Force

Write-Host "UI assets built successfully in the dist directory"
exit 0 # Explicitly return success exit code 