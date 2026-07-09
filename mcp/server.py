from mcp.server.fastmcp import FastMCP

game_mcp = FastMCP("Demo")

@game_mcp.tool()
def get_game_today() -> str:
    """今日やったゲーム"""
    return "stray"

if __name__ == "__main__":
    game_mcp.run()
