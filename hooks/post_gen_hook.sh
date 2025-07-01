#!/bin/sh
# This hook script runs after the project is generated.

# 綠色
GREEN='\033[0;32m'
# 無顏色
NC='\033[0m'

echo ""
echo "${GREEN}✔ Project '{{ project_name }}' generated successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. cd {{ project_name }}"
echo "  2. Review the .env.example file and create your own .env file."
echo "     (cp .env.example .env)"
echo "  3. Set up your local database or use Docker:"
echo "     docker-compose up -d"
echo "  4. Run the application:"
echo "     cargo run"
echo ""
echo "Happy coding! ✨"