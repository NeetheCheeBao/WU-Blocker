import os
import sys
import ctypes
import datetime
import subprocess
import tkinter as tk
from tkinter import ttk
import winreg
import webbrowser

# 定义注册表路径
REG_PATH = r"SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"

# 与「暂停更新」相关的注册表值
PAUSE_VALUES = [
    "FlightSettingsMaxPauseDays",
    "PauseFeatureUpdatesStartTime",
    "PauseFeatureUpdatesEndTime",
    "PauseQualityUpdatesStartTime",
    "PauseQualityUpdatesEndTime",
    "PauseUpdatesStartTime",
    "PauseUpdatesExpiryTime",
]

# 终端配色
TERM_COLORS = {
    "info": "#D4D4D4",
    "ok":   "#4EC9B0",
    "warn": "#DCDCAA",
    "err":  "#F48771",
    "cmd":  "#569CD6",
    "dim":  "#7A7A7A",
}

term = None                # 终端控件，在 UI 初始化后赋值
output_queue = []          # 待输出行队列
is_draining = False        # 是否正在逐行输出
DRAIN_DELAY_MS = 5         # 每行输出之间的间隔（毫秒）


# 资源路径处理，兼容 PyInstaller 打包
def resource_path(relative_path):
    """获取资源的绝对路径，兼容开发环境和 PyInstaller 打包后的单文件"""
    try:
        base_path = sys._MEIPASS
    except AttributeError:
        base_path = os.path.abspath(".")
    return os.path.join(base_path, relative_path)


# 权限与 DPI 相关函数
def is_admin():
    """检查当前是否具有管理员权限"""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except Exception:
        return False


def run_as_admin():
    """尝试以管理员权限重新启动脚本"""
    params = " ".join(f'"{arg}"' for arg in sys.argv)
    ctypes.windll.shell32.ShellExecuteW(
        None, "runas", sys.executable, params, None, 1
    )


def set_dpi_awareness():
    """设置 DPI 感知，防止高分屏下界面模糊"""
    try:
        # Windows 8.1 及以上版本
        ctypes.windll.shcore.SetProcessDpiAwareness(1)
    except Exception:
        try:
            # Windows Vista/7/8
            ctypes.windll.user32.SetProcessDPIAware()
        except Exception:
            pass


def enable_window_shadow(root):
    """通过 CS_DROPSHADOW 类样式为窗口启用阴影"""
    try:
        hwnd = ctypes.windll.user32.GetParent(root.winfo_id())
        if not hwnd:
            hwnd = root.winfo_id()
        GCL_STYLE = -26
        CS_DROPSHADOW = 0x00020000
        style = ctypes.windll.user32.GetClassLongW(hwnd, GCL_STYLE)
        ctypes.windll.user32.SetClassLongW(hwnd, GCL_STYLE, style | CS_DROPSHADOW)
    except Exception:
        pass


def enable_taskbar_visibility(root):
    """为无边框窗口添加 WS_EX_APPWINDOW 样式，使其在任务栏显示"""
    try:
        root.withdraw()
        root.update_idletasks()

        hwnd = ctypes.windll.user32.GetParent(root.winfo_id())
        if not hwnd:
            hwnd = root.winfo_id()

        GWL_EXSTYLE = -20
        WS_EX_APPWINDOW = 0x00040000
        WS_EX_TOOLWINDOW = 0x00000080

        ex_style = ctypes.windll.user32.GetWindowLongW(hwnd, GWL_EXSTYLE)
        ex_style = (ex_style & ~WS_EX_TOOLWINDOW) | WS_EX_APPWINDOW
        ctypes.windll.user32.SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style)

        root.deiconify()
    except Exception:
        # 若失败，至少保证窗口处于显示状态
        try:
            root.deiconify()
        except Exception:
            pass


# 系统信息读取
class RTL_OSVERSIONINFOW(ctypes.Structure):
    """RtlGetVersion 所需的结构体"""
    _fields_ = [
        ("dwOSVersionInfoSize", ctypes.c_ulong),
        ("dwMajorVersion",      ctypes.c_ulong),
        ("dwMinorVersion",      ctypes.c_ulong),
        ("dwBuildNumber",       ctypes.c_ulong),
        ("dwPlatformId",        ctypes.c_ulong),
        ("szCSDVersion",        ctypes.c_wchar * 128),
    ]


def get_nt_version():
    """通过 ntdll.RtlGetVersion 获取真实的 NT 版本号（不受兼容模式影响）"""
    try:
        version = RTL_OSVERSIONINFOW()
        version.dwOSVersionInfoSize = ctypes.sizeof(version)
        ctypes.windll.ntdll.RtlGetVersion(ctypes.byref(version))
        return version.dwMajorVersion, version.dwMinorVersion, version.dwBuildNumber
    except Exception:
        return 0, 0, 0


def get_os_info():
    """获取真实的 Windows 产品名称、版本号（带 build 号）"""
    major, minor, build = get_nt_version()

    # 从注册表读取产品名称和显示版本
    product_name = ""
    display_version = ""
    try:
        key = winreg.OpenKey(
            winreg.HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            0,
            winreg.KEY_READ,
        )
        try:
            product_name, _ = winreg.QueryValueEx(key, "ProductName")
        except FileNotFoundError:
            product_name = ""
        for name in ("DisplayVersion", "ReleaseId"):
            try:
                display_version, _ = winreg.QueryValueEx(key, name)
                break
            except FileNotFoundError:
                continue
        try:
            edition, _ = winreg.QueryValueEx(key, "EditionID")
        except FileNotFoundError:
            edition = ""
        winreg.CloseKey(key)
    except Exception:
        edition = ""

    # 修正：Win10 与 Win11 共享主版本号，用 build 号区分
    if major == 10 and build >= 22000:
        product_name = "Windows 11"
    elif major == 10 and not product_name:
        product_name = "Windows 10"

    if not product_name:
        product_name = f"Windows NT {major}.{minor}"

    # 组装：产品名称 + 版本 + Build
    parts = [product_name]
    if display_version:
        parts.append(display_version)
    if build:
        parts.append(f"Build {build}")
    return " ".join(parts)


def get_privilege_text():
    """返回真实的权限状态文本"""
    return "管理员" if is_admin() else "普通用户"


# 终端输出相关函数
def term_write(text="", tag="info"):
    """将一行文本加入输出队列，由 _drain_one 逐行显示"""
    timestamp = datetime.datetime.now().strftime("[%H:%M:%S]  ")
    output_queue.append((timestamp + text, tag))
    _start_drain()


def _start_drain():
    """如果当前没有输出任务，启动逐行输出"""
    global is_draining
    if is_draining or not output_queue:
        return
    is_draining = True
    _drain_one()


def _drain_one():
    """输出队列中的一行，并计划下一行"""
    global is_draining
    if term is None or not output_queue:
        is_draining = False
        return

    text, tag = output_queue.pop(0)
    term.config(state=tk.NORMAL)
    term.insert(tk.END, text + "\n", tag)

    # 限制终端最多 200 行，超出则删除旧行
    total_lines = int(term.index('end-1c').split('.')[0])
    if total_lines > 200:
        lines_to_remove = total_lines - 200
        term.delete("1.0", f"{lines_to_remove + 1}.0")

    term.see(tk.END)
    term.config(state=tk.DISABLED)

    term.after(DRAIN_DELAY_MS, _drain_one)


def clear_term():
    """立即清空终端显示和待输出队列"""
    output_queue.clear()
    if term is None:
        return
    term.config(state=tk.NORMAL)
    term.delete("1.0", tk.END)
    term.config(state=tk.DISABLED)


# 注册表读取相关函数
def read_pause_settings():
    """读取注册表，返回 (键是否存在, {值名: 数据})"""
    values = {}
    try:
        key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, REG_PATH, 0, winreg.KEY_READ)
    except FileNotFoundError:
        return False, values

    try:
        for name in PAUSE_VALUES:
            try:
                data, _ = winreg.QueryValueEx(key, name)
                values[name] = data
            except FileNotFoundError:
                continue
    finally:
        winreg.CloseKey(key)

    return True, values


def parse_time(value):
    """把 2000-01-01T01:00:00Z 解析成 datetime，失败返回 None"""
    if not isinstance(value, str):
        return None
    try:
        return datetime.datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError:
        return None


def pretty_time(value):
    """格式化时间为可读字符串"""
    dt = parse_time(value) if isinstance(value, str) else value
    if dt is None:
        return str(value)
    return dt.strftime("%Y-%m-%d %H:%M:%S UTC")


def check_update_status():
    """读取注册表并把当前更新状态输出到终端"""
    try:
        exists, values = read_pause_settings()
    except OSError:
        return

    if not exists or not values:
        term_write("当前状态: 自动更新正常运行。", "warn")
        return

    now = datetime.datetime.now(datetime.timezone.utc).replace(tzinfo=None)
    expiry = parse_time(values.get("PauseUpdatesExpiryTime"))
    start = values.get("PauseUpdatesStartTime")

    if expiry and expiry > now:
        days = (expiry - now).days
        start_str = pretty_time(start) if start else "2000-01-01 01:00:00 UTC"
        expiry_str = pretty_time(expiry)

        term_write("当前状态: 自动更新已被暂停。", "ok")
        term_write(f"暂停开始时间 : {start_str}", "ok")
        term_write(f"暂停到期时间 : {expiry_str}", "ok")
        term_write(f"剩余时间     : 约 {days:,} 天", "ok")
        if expiry.year >= 9999:
            term_write("(到期时间为 9999 年，等同于永久暂停)", "dim")
    else:
        term_write("当前状态: 自动更新正常运行。", "warn")


# 操作相关函数
def print_system_info():
    """输出软件名称、系统信息和权限，然后显示更新状态"""
    term_write("Windows Update Blocker v1.0.1", "cmd")
    term_write(get_os_info(), "info")
    term_write(f"程序权限：{get_privilege_text()}", "info")
    check_update_status()


def refresh_status():
    """手动刷新状态：清空终端，首行显示刷新提示，再输出系统信息和状态"""
    clear_term()
    term_write("> 手动刷新当前状态", "cmd")
    print_system_info()


def pause_updates():
    """暂停更新至 9999-12-31"""
    term_write("-" * 40, "dim")
    term_write("> 执行操作: 永久暂停更新", "cmd")

    try:
        key = winreg.CreateKeyEx(winreg.HKEY_LOCAL_MACHINE, REG_PATH, 0, winreg.KEY_WRITE)
        # 0xFFFFFFFF 为 DWORD 最大值，相当于允许的最长暂停天数
        winreg.SetValueEx(key, "FlightSettingsMaxPauseDays", 0, winreg.REG_DWORD, 4294967295)

        time_start = "2000-01-01T01:00:00Z"
        time_end = "9999-12-31T01:00:00Z"

        values_to_set = {
            "PauseFeatureUpdatesStartTime": time_start,
            "PauseFeatureUpdatesEndTime": time_end,
            "PauseQualityUpdatesStartTime": time_start,
            "PauseQualityUpdatesEndTime": time_end,
            "PauseUpdatesStartTime": time_start,
            "PauseUpdatesExpiryTime": time_end,
        }

        for name, value in values_to_set.items():
            winreg.SetValueEx(key, name, 0, winreg.REG_SZ, value)

        winreg.CloseKey(key)

        term_write("[√] 注册表写入成功。", "ok")
        term_write("[√] Windows 自动更新已暂停，有效期至 9999-12-31。", "ok")

    except Exception as e:
        term_write(f"[错误] 无法修改注册表: {e}", "err")
        term_write("[提示] 请确保以管理员身份运行本程序。", "warn")

    term_write("-" * 40, "dim")
    term_write("> 刷新当前更新状态:", "cmd")
    check_update_status()


def restore_updates():
    """恢复自动更新"""
    term_write("-" * 40, "dim")
    term_write("> 执行操作: 恢复自动更新", "cmd")

    try:
        try:
            key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, REG_PATH, 0, winreg.KEY_WRITE)
        except FileNotFoundError:
            term_write("[*] 注册表项不存在，系统已处于默认更新状态。", "warn")
            term_write("-" * 40, "dim")
            term_write("> 刷新当前更新状态:", "cmd")
            check_update_status()
            return

        removed = 0
        for value_name in PAUSE_VALUES:
            try:
                winreg.DeleteValue(key, value_name)
                removed += 1
            except FileNotFoundError:
                pass

        winreg.CloseKey(key)

        term_write(f"[√] 已清除 {removed} 项暂停设置。", "ok")
        term_write("[√] Windows 自动更新已恢复默认设置。", "ok")

    except Exception as e:
        term_write(f"[错误] 操作失败: {e}", "err")

    term_write("-" * 40, "dim")
    term_write("> 刷新当前更新状态:", "cmd")
    check_update_status()


def open_github():
    """检查更新：跳转到项目主页"""
    webbrowser.open("https://github.com/NeetheCheeBao/WU-Blocker")


def _last_key_prefix():
    """根据系统 UI 语言返回注册表编辑器中「我的电脑」的显示名称"""
    try:
        lang = ctypes.windll.kernel32.GetUserDefaultUILanguage()
        if (lang & 0x3FF) == 0x04:      # 中文（简体 / 繁体）
            return "计算机"
    except Exception:
        pass
    return "Computer"


def open_regedit():
    """打开注册表编辑器，并定位到 Windows Update 设置项"""
    term_write("> 打开注册表编辑器并定位到:", "cmd")
    term_write(r"  HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings", "dim")

    last_key = (
        _last_key_prefix()
        + r"\HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"
    )

    # 先写入 LastKey，regedit 启动时会自动跳转到该位置
    try:
        key = winreg.CreateKeyEx(
            winreg.HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Applets\Regedit",
            0,
            winreg.KEY_WRITE,
        )
        winreg.SetValueEx(key, "LastKey", 0, winreg.REG_SZ, last_key)
        winreg.CloseKey(key)
    except Exception as e:
        term_write(f"[提示] 无法预设注册表定位信息: {e}", "warn")

    try:
        # /m 可让已打开的 regedit 另开一个窗口，避免只激活旧窗口
        subprocess.Popen(["regedit.exe", "/m"])
        term_write("[√] 已启动注册表编辑器。", "ok")
    except Exception as e:
        term_write(f"[错误] 无法启动注册表编辑器: {e}", "err")


def show_startup_info():
    """启动时输出程序信息及状态"""
    print_system_info()


# 主程序入口
if __name__ == "__main__":
    set_dpi_awareness()

    if not is_admin():
        run_as_admin()
        sys.exit()

    root = tk.Tk()
    root.title("WU-Blocker v1.0.1")

    # 设置窗口图标，兼容 PyInstaller 打包
    icon_path = resource_path("icon.ico")
    try:
        root.iconbitmap(icon_path)
    except Exception:
        pass

    root.overrideredirect(True)          # 隐藏系统标题栏
    root.attributes("-topmost", True)    # 窗口置顶，防止被其他窗口遮挡
    root.configure(bg="#F0F0F0")         # 窗体背景色

    # 让无边框窗口在任务栏中显示（需要在主循环前调用）
    enable_taskbar_visibility(root)

    # 计算 DPI 缩放
    try:
        current_dpi = ctypes.windll.user32.GetDpiForWindow(root.winfo_id())
        scale_factor = current_dpi / 96.0
    except Exception:
        scale_factor = 1.0

    if scale_factor <= 1.0:
        try:
            scale_factor = root.winfo_fpixels("1i") / 96.0
        except Exception:
            scale_factor = 1.0

    def sc(value):
        return int(round(value * scale_factor))

    base_width, base_height = 460, 300
    screen_width = root.winfo_screenwidth()
    screen_height = root.winfo_screenheight()
    x = (screen_width - sc(base_width)) // 2
    y = (screen_height - sc(base_height)) // 2
    root.geometry(f"{sc(base_width)}x{sc(base_height)}+{x}+{y}")

    # 启用窗口阴影
    enable_window_shadow(root)

    # 样式设置
    style = ttk.Style()
    style.theme_use("clam")

    base_font = ("Microsoft YaHei", max(9, sc(10)))
    term_font = ("Consolas", max(8, sc(9)))

    # 所有 Frame 使用与窗体一致的颜色
    style.configure("TFrame", background="#F0F0F0")

    # 按钮基础样式
    style.configure(
        "TButton",
        font=base_font,
        padding=2,
        background="#F5F5F5",
        foreground="black",
        borderwidth=1,
        relief="groove",
    )

    # 悬停时背景 #E0EEF9，边框 #1D86D7；按下时颜色略深
    style.map(
        "TButton",
        background=[
            ("pressed", "#C7E2F5"),
            ("active",  "#E0EEF9"),
        ],
        foreground=[
            ("pressed", "black"),
            ("active",  "black"),
        ],
        bordercolor=[
            ("pressed", "#1D86D7"),
            ("active",  "#1D86D7"),
        ],
    )

    # 主框架
    main_frame = ttk.Frame(root, padding=(sc(4), sc(4), sc(4), sc(6)))
    main_frame.pack(fill=tk.BOTH, expand=True)

    # 实现拖动窗口
    def start_move(event):
        root.x = event.x
        root.y = event.y

    def stop_move(event):
        root.x = None
        root.y = None

    def on_move(event):
        if hasattr(root, 'x') and root.x is not None and root.y is not None:
            deltax = event.x - root.x
            deltay = event.y - root.y
            new_x = root.winfo_x() + deltax
            new_y = root.winfo_y() + deltay
            root.geometry(f"+{new_x}+{new_y}")

    def bind_drag(widget):
        """为控件绑定窗口拖动事件"""
        widget.bind("<ButtonPress-1>", start_move)
        widget.bind("<ButtonRelease-1>", stop_move)
        widget.bind("<B1-Motion>", on_move)

    # 主框架可拖动
    bind_drag(main_frame)

    # 按 Esc 键退出程序
    root.bind("<Escape>", lambda e: root.destroy())

    # 内嵌终端区域（底部留出更多间距，使终端与分隔线之间更空）
    term_frame = ttk.Frame(main_frame)
    term_frame.pack(fill=tk.BOTH, expand=True, pady=(0, sc(27)))

    term = tk.Text(
        term_frame,
        bg="#0C0C0C",
        fg=TERM_COLORS["info"],
        font=term_font,
        relief=tk.FLAT,
        wrap=tk.WORD,
        height=8,
        padx=sc(8),
        pady=sc(8),
        state=tk.DISABLED,
        highlightthickness=1,
        highlightbackground="#3C3C3C",
        highlightcolor="#3C3C3C",
        cursor="arrow",
        insertwidth=0,
    )

    # 不使用滚动条，直接填满终端区域
    term.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

    for tag_name, color in TERM_COLORS.items():
        term.tag_config(tag_name, foreground=color)

    # 终端与按钮之间的分割线
    sep = ttk.Separator(main_frame, orient="horizontal")
    sep.pack(fill=tk.X, pady=(0, sc(12)))
    # 分隔线也可拖动窗口
    bind_drag(sep)

    # 底部按钮区域
    btn_frame = ttk.Frame(main_frame)
    btn_frame.pack(fill=tk.X)

    btn_refresh = ttk.Button(btn_frame, text="刷新", width=6, command=refresh_status)
    btn_refresh.pack(side=tk.LEFT, padx=(0, sc(4)))

    btn_pause = ttk.Button(btn_frame, text="永久暂停更新", command=pause_updates)
    btn_pause.pack(side=tk.LEFT, expand=True, fill=tk.X, padx=(sc(4), sc(4)))

    btn_restore = ttk.Button(btn_frame, text="恢复自动更新", command=restore_updates)
    btn_restore.pack(side=tk.LEFT, expand=True, fill=tk.X, padx=(sc(4), sc(4)))

    btn_exit = ttk.Button(btn_frame, text="退出", width=6, command=root.destroy)
    btn_exit.pack(side=tk.LEFT, padx=(sc(4), sc(4)))

    # ---- 汉堡菜单 ----
    def show_menu():
        """在按钮正下方弹出悬浮菜单"""
        x = btn_menu.winfo_rootx() + btn_menu.winfo_width() - app_menu.winfo_reqwidth()
        y = btn_menu.winfo_rooty() + btn_menu.winfo_height()
        try:
            app_menu.tk_popup(x, y)
        finally:
            app_menu.grab_release()

    # 用固定尺寸的容器把菜单按钮撑成正方形
    menu_size = sc(28)
    menu_box = ttk.Frame(btn_frame, width=menu_size, height=menu_size)
    menu_box.pack(side=tk.LEFT, padx=(sc(4), 0))
    menu_box.pack_propagate(False)   # 禁止子控件改变容器尺寸

    btn_menu = ttk.Button(menu_box, text="≡", width=1, command=show_menu)
    btn_menu.place(x=0, y=0, relwidth=1, relheight=1)

    app_menu = tk.Menu(
        root,
        tearoff=0,
        font=base_font,
        bg="#FFFFFF",
        fg="black",
        activebackground="#E0EEF9",
        activeforeground="black",
        borderwidth=1,
        relief="solid",
    )
    app_menu.add_command(label="打开注册表", command=open_regedit)
    app_menu.add_command(label="检查更新", command=open_github)

    # 启动后自动输出程序信息及状态
    root.after(120, show_startup_info)

    root.mainloop()