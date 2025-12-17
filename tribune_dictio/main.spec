# -*- mode: python ; coding: utf-8 -*-

from PyInstaller.utils.hooks import collect_all

# collect all torch files
torch_datas, torch_binaries, torch_hiddenimports = collect_all('torch')
torchaudio_datas, torchaudio_binaries, torchaudio_hiddenimports = collect_all('torchaudio')



a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)


a.datas += torch_datas + torchaudio_datas
a.binaries += torch_binaries + torchaudio_binaries
a.hiddenimports += torch_hiddenimports + torchaudio_hiddenimports

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='main',
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
