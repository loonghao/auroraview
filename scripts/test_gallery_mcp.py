#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Gallery MCP Server 测试脚本。

用于在不启动 IDE 的情况下测试 Gallery 的嵌入式 MCP Server。

Usage:
    # 测试指定端口的 MCP Server
    python scripts/test_gallery_mcp.py --port 27168

    # 测试所有工具
    python scripts/test_gallery_mcp.py --port 27168 --all

    # 测试特定工具
    python scripts/test_gallery_mcp.py --port 27168 --tool api.get_samples

    # 列出所有可用工具
    python scripts/test_gallery_mcp.py --port 27168 --list
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.request
import urllib.error
from typing import Any


class McpTester:
    """MCP Server 测试客户端。"""

    def __init__(self, host: str = "127.0.0.1", port: int = 27168):
        self.host = host
        self.port = port
        self.base_url = f"http://{host}:{port}"
        self.request_id = 0

    def _next_id(self) -> int:
        self.request_id += 1
        return self.request_id

    def _send_mcp_request(self, method: str, params: dict | None = None) -> dict:
        """发送 MCP JSON-RPC 请求。"""
        payload = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
            "params": params or {},
        }

        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self.base_url}/mcp",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                response_text = resp.read().decode("utf-8")
                
                # 尝试直接解析 JSON
                try:
                    return json.loads(response_text)
                except json.JSONDecodeError:
                    pass

                # 解析 SSE 格式响应
                for line in response_text.split("\n"):
                    if line.startswith("data: "):
                        return json.loads(line[6:])
                return {"error": "No data in response", "raw": response_text}
        except urllib.error.URLError as e:
            return {"error": str(e)}
        except json.JSONDecodeError as e:
            return {"error": f"JSON decode error: {e}", "raw": response_text}

    def health_check(self) -> dict:
        """检查 MCP Server 健康状态。"""
        try:
            req = urllib.request.Request(f"{self.base_url}/health")
            with urllib.request.urlopen(req, timeout=5) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.URLError as e:
            return {"error": str(e), "status": "unhealthy"}

    def initialize(self) -> dict:
        """初始化 MCP 会话。"""
        return self._send_mcp_request(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "gallery-mcp-tester", "version": "1.0.0"},
            },
        )

    def list_tools(self) -> dict:
        """列出所有可用工具。"""
        return self._send_mcp_request("tools/list", {})

    def call_tool(self, name: str, arguments: dict | None = None) -> dict:
        """调用指定工具。"""
        return self._send_mcp_request(
            "tools/call",
            {"name": name, "arguments": arguments or {}},
        )


def print_result(title: str, result: Any, indent: int = 2):
    """格式化打印结果。"""
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print("=" * 60)
    if isinstance(result, dict):
        print(json.dumps(result, indent=indent, ensure_ascii=False))
    else:
        print(result)


def test_health(tester: McpTester) -> bool:
    """测试健康端点。"""
    result = tester.health_check()
    print_result("Health Check", result)
    return "error" not in result


def test_initialize(tester: McpTester) -> bool:
    """测试 MCP 初始化。"""
    result = tester.initialize()
    print_result("MCP Initialize", result)
    return "result" in result


def test_list_tools(tester: McpTester) -> list[str]:
    """测试列出工具。"""
    result = tester.list_tools()
    print_result("List Tools", result)

    tools = []
    if "result" in result and "tools" in result["result"]:
        tools = [t["name"] for t in result["result"]["tools"]]
        print(f"\n发现 {len(tools)} 个工具:")
        for tool in sorted(tools):
            print(f"  - {tool}")
    return tools


def test_tool(tester: McpTester, tool_name: str, arguments: dict | None = None) -> dict:
    """测试调用指定工具。"""
    result = tester.call_tool(tool_name, arguments)
    
    # 尝试解析内部结果以便更好地验证
    if "result" in result and "content" in result["result"]:
        try:
             content = result["result"]["content"]
             if content and isinstance(content, list) and "text" in content[0]:
                 text = content[0]["text"]
                 inner = json.loads(text)
                 # 更新结果以便更容易调试
                 result["_parsed"] = inner
        except Exception:
             pass

    print_result(f"Call Tool: {tool_name}", result)
    return result


def run_all_tests(tester: McpTester):
    """运行所有测试。"""
    print("\n" + "=" * 60)
    print("  Gallery MCP Server 完整测试")
    print("=" * 60)

    # 1. 健康检查
    if not test_health(tester):
        print("\n❌ 健康检查失败，MCP Server 可能未运行")
        return False

    # 2. 初始化
    if not test_initialize(tester):
        print("\n❌ MCP 初始化失败")
        return False

    # 3. 列出工具
    tools = test_list_tools(tester)
    if not tools:
        print("\n⚠️ 未发现任何工具")

    # 4. 测试核心工具
    core_tests = [
        ("api.get_samples", {}),
        ("api.get_categories", {}),
        # ("api.get_mcp_info", {}), # Removed from main.py, check if it's in list first
    ]
    if "api.get_mcp_info" in tools:
        core_tests.append(("api.get_mcp_info", {}))

    print("\n" + "-" * 60)
    print("  测试核心工具")
    print("-" * 60)

    success_count = 0
    for tool_name, args in core_tests:
        if tool_name in tools:
            result = test_tool(tester, tool_name, args)
            if "result" in result:
                # 检查信封格式
                parsed = result.get("_parsed")
                if parsed and isinstance(parsed, dict) and "ok" in parsed:
                    if parsed["ok"]:
                        success_count += 1
                        print(f"  ✅ {tool_name}")
                    else:
                        print(f"  ❌ {tool_name}: API Error - {parsed.get('error')}")
                else:
                    # 兼容旧格式或非 JSON 返回
                    success_count += 1
                    print(f"  ✅ {tool_name} (Raw/Unknown format)")
            else:
                print(f"  ❌ {tool_name}: MCP Error - {result.get('error', 'Unknown error')}")
        else:
            print(f"  ⚠️ {tool_name}: 工具不存在")

    # 5. 测试带参数的工具
    if "api.get_source" in tools:
        print("\n" + "-" * 60)
        print("  测试带参数的工具")
        print("-" * 60)
        result = test_tool(tester, "api.get_source", {"sample_id": "hello_world"})
        if "result" in result:
             parsed = result.get("_parsed")
             if parsed and isinstance(parsed, dict) and "ok" in parsed:
                 if parsed["ok"]:
                    success_count += 1
                    print("  ✅ api.get_source")
                 else:
                    print(f"  ❌ api.get_source: API Error - {parsed.get('error')}")
             else:
                success_count += 1
                print("  ✅ api.get_source (Raw/Unknown format)")
        else:
            print(f"  ❌ api.get_source: MCP Error - {result.get('error', 'Unknown error')}")

    # 总结
    print("\n" + "=" * 60)
    print(f"  测试完成: {success_count}/{len(core_tests) + 1} 通过")
    print("=" * 60)

    return success_count > 0


def main():
    parser = argparse.ArgumentParser(description="Gallery MCP Server 测试工具")
    parser.add_argument("--host", default="127.0.0.1", help="MCP Server 主机 (默认: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=27168, help="MCP Server 端口 (默认: 27168)")
    parser.add_argument("--all", action="store_true", help="运行所有测试")
    parser.add_argument("--list", action="store_true", help="列出所有可用工具")
    parser.add_argument("--tool", type=str, help="测试指定工具")
    parser.add_argument("--args", type=str, default="{}", help="工具参数 (JSON 格式)")
    parser.add_argument("--health", action="store_true", help="仅测试健康端点")

    args = parser.parse_args()

    tester = McpTester(host=args.host, port=args.port)
    print(f"目标: {tester.base_url}")

    if args.health:
        test_health(tester)
    elif args.list:
        test_initialize(tester)
        test_list_tools(tester)
    elif args.tool:
        test_initialize(tester)
        tool_args = json.loads(args.args)
        test_tool(tester, args.tool, tool_args)
    elif args.all:
        run_all_tests(tester)
    else:
        # 默认：健康检查 + 初始化 + 列出工具
        if test_health(tester):
            test_initialize(tester)
            test_list_tools(tester)
        else:
            print("\n提示: 请确保 Gallery 正在运行")
            print("  启动命令: just gallery")
            print(f"  或指定端口: AURORAVIEW_MCP_PORT={args.port} just gallery")


if __name__ == "__main__":
    main()
