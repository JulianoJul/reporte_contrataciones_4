# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec for data_service.exe
Build: pyinstaller python/data_service.spec
"""

block_cipher = None

a = Analysis(
    ['data_service.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[
        'pandas',
        'pandas._libs.tslibs.np_datetime',
        'pandas._libs.tslibs.parsing',
        'pandas._libs.tslibs.timedeltas',
        'pandas._libs.tslibs.timestamps',
        'pandas._libs.tslibs.timezones',
        'openpyxl',
        'openpyxl.cell._writer',
        'fpdf',
        'pptx',
        'sqlite3',
        'queries',
        'exporters',
        'lxml',
        'et_xmlfile',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        'tkinter',
        'matplotlib',
        'scipy',
        'PIL',
        'cv2',
        'PyQt5',
        'PySide2',
        'notebook',
        'IPython',
    ],
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='data_service',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
