import sys
import ctypes
import tkinter as tk
from tkinter import ttk, messagebox
import winreg
import webbrowser

# 定义注册表路径
REG_PATH = r"SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"

def is_admin():
    """检查当前是否具有管理员权限"""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_as_admin():
    """尝试以管理员权限重新启动脚本"""
    ctypes.windll.shell32.ShellExecuteW(
        None, "runas", sys.executable, " ".join(sys.argv), None, 1
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

def pause_updates():
    """暂停更新至 9999-12-31"""
    try:
        key = winreg.CreateKeyEx(winreg.HKEY_LOCAL_MACHINE, REG_PATH, 0, winreg.KEY_WRITE)
        winreg.SetValueEx(key, "FlightSettingsMaxPauseDays", 0, winreg.REG_DWORD, 7000)
        
        time_start = "2000-01-01T01:00:00Z"
        time_end = "9999-12-31T01:00:00Z"
        
        values_to_set = {
            "PauseFeatureUpdatesStartTime": time_start,
            "PauseFeatureUpdatesEndTime": time_end,
            "PauseQualityUpdatesStartTime": time_start,
            "PauseQualityUpdatesEndTime": time_end,
            "PauseUpdatesStartTime": time_start,
            "PauseUpdatesExpiryTime": time_end
        }
        
        for name, value in values_to_set.items():
            winreg.SetValueEx(key, name, 0, winreg.REG_SZ, value)
            
        winreg.CloseKey(key)
        
        status_label.config(text="状态: 已成功暂停更新至 9999年", foreground="green")
        messagebox.showinfo("成功", "Windows 自动更新已暂停！\n\n有效期至：9999-12-31")
        
    except Exception as e:
        status_label.config(text=str(e), foreground="red")
        messagebox.showerror("错误", f"无法修改注册表。\n请确保以管理员身份运行。\n错误信息: {e}")

def restore_updates():
    """恢复自动更新"""
    try:
        try:
            key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, REG_PATH, 0, winreg.KEY_WRITE)
        except FileNotFoundError:
            status_label.config(text="状态: 系统已处于默认更新状态", foreground="blue")
            messagebox.showinfo("提示", "Windows 更新设置已是默认状态。")
            return

        values_to_delete = [
            "FlightSettingsMaxPauseDays",
            "PauseFeatureUpdatesStartTime",
            "PauseFeatureUpdatesEndTime",
            "PauseQualityUpdatesStartTime",
            "PauseQualityUpdatesEndTime",
            "PauseUpdatesStartTime",
            "PauseUpdatesExpiryTime"
        ]
        
        for value_name in values_to_delete:
            try:
                winreg.DeleteValue(key, value_name)
            except FileNotFoundError:
                pass
                
        winreg.CloseKey(key)
        status_label.config(text="状态: 已恢复自动更新", foreground="blue")
        messagebox.showinfo("成功", "Windows 自动更新已恢复默认设置！")
        
    except Exception as e:
        status_label.config(text=str(e), foreground="red")
        messagebox.showerror("错误", f"操作失败: {e}")

def open_github():
    webbrowser.open("https://github.com/NeetheCheeBao/WU-Blocker")

# --- 主程序逻辑 ---
if __name__ == "__main__":
    set_dpi_awareness()

    if not is_admin():
        run_as_admin()
        sys.exit()

    root = tk.Tk()
    root.title("WU-Blocker v1.0.0 by NeetheCheeBao")
    
    try:
        current_dpi = ctypes.windll.user32.GetDpiForWindow(root.winfo_id())
        scale_factor = current_dpi / 96.0
    except:
        scale_factor = 1.0

    base_width = 400
    base_height = 300
    window_width = int(base_width * scale_factor)
    window_height = int(base_height * scale_factor)
    
    root.geometry(f"{window_width}x{window_height}")
    
    root.resizable(True, True)

    style = ttk.Style()
    style.theme_use('clam')
    
    base_font = ('Microsoft YaHei', 10)
    header_font = ('Microsoft YaHei', 12, 'bold')
    
    style.configure('TButton', font=base_font, padding=5)
    style.configure('Header.TLabel', font=header_font)
    style.configure('Status.TLabel', font=('Microsoft YaHei', 9))

    menubar = tk.Menu(root)
    menubar.add_command(label="项目地址", command=open_github)
    root.config(menu=menubar)

    main_frame = ttk.Frame(root, padding=int(20 * scale_factor))
    main_frame.pack(fill=tk.BOTH, expand=True)

    title_label = ttk.Label(main_frame, text="管控Windows自动更新", style='Header.TLabel')
    title_label.pack(pady=(0, int(20 * scale_factor)))

    desc_label = ttk.Label(main_frame, text="请选择您要执行的操作：", font=base_font)
    desc_label.pack(pady=(0, int(10 * scale_factor)))

    btn_pause = ttk.Button(main_frame, text="永久暂停更新 (至9999年)", command=pause_updates)
    btn_pause.pack(pady=int(10 * scale_factor), ipady=5, fill=tk.X, padx=20)

    btn_restore = ttk.Button(main_frame, text="恢复自动更新", command=restore_updates)
    btn_restore.pack(pady=int(10 * scale_factor), ipady=5, fill=tk.X, padx=20)

    status_label = ttk.Label(main_frame, text="就绪", style='Status.TLabel', foreground="gray")
    status_label.pack(side=tk.BOTTOM, pady=(int(20 * scale_factor), 0))

    root.update_idletasks()
    x = (root.winfo_screenwidth() // 2) - (root.winfo_width() // 2)
    y = (root.winfo_screenheight() // 2) - (root.winfo_height() // 2)
    root.geometry(f'+{x}+{y}')

    root.mainloop()