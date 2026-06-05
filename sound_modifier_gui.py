#!/usr/bin/env python3
import os
import sys
import json
import copy
import shutil
import datetime
import xml.etree.ElementTree as ET

# Ensure PySide6 or PyQt6 is available
try:
    from PySide6.QtCore import Qt, QSize, QLocale
    from PySide6.QtGui import QFont, QDoubleValidator
    from PySide6.QtWidgets import (
        QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
        QTabWidget, QLabel, QLineEdit, QPushButton, QFileDialog, QGroupBox,
        QScrollArea, QFormLayout, QComboBox, QCheckBox, QListWidget, QSplitter,
        QMessageBox, QDoubleSpinBox, QProgressBar, QStatusBar, QGridLayout,
        QFrame, QSizePolicy, QTextEdit, QInputDialog
    )
    PYSIDE = True
except ImportError:
    try:
        from PyQt6.QtCore import Qt, QSize, QLocale
        from PyQt6.QtGui import QFont, QDoubleValidator
        from PyQt6.QtWidgets import (
            QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
            QTabWidget, QLabel, QLineEdit, QPushButton, QFileDialog, QGroupBox,
            QScrollArea, QFormLayout, QComboBox, QCheckBox, QListWidget, QSplitter,
            QMessageBox, QDoubleSpinBox, QProgressBar, QStatusBar, QGridLayout,
            QFrame, QSizePolicy, QTextEdit, QInputDialog
        )
        PYSIDE = False
    except ImportError:
        print("Neither PySide6 nor PyQt6 is installed. Please install one of them.", file=sys.stderr)
        sys.exit(1)

# Helper to adapt Qt properties between PySide6 and PyQt6
def get_align_left():
    return Qt.AlignmentFlag.AlignLeft if hasattr(Qt, "AlignmentFlag") else Qt.AlignLeft

def get_align_center():
    return Qt.AlignmentFlag.AlignCenter if hasattr(Qt, "AlignmentFlag") else Qt.AlignTop # AlignCenter fallback

# Style sheet for dark mode functional layout
DARK_STYLE = """
QMainWindow {
    background-color: #1e1e24;
    color: #e0e0e6;
}
QTabWidget::pane {
    border: 1px solid #3c3c44;
    background-color: #1e1e24;
}
QTabBar::tab {
    background: #2b2b33;
    border: 1px solid #3c3c44;
    padding: 8px 16px;
    color: #c0c0c6;
    font-weight: bold;
}
QTabBar::tab:selected {
    background: #3c3c44;
    color: #ffffff;
    border-bottom-color: #1e1e24;
}
QGroupBox {
    border: 1px solid #3c3c44;
    border-radius: 6px;
    margin-top: 12px;
    font-weight: bold;
    color: #ffffff;
    padding: 10px;
}
QGroupBox::title {
    subcontrol-origin: margin;
    left: 8px;
    padding: 0 3px 0 3px;
}
QLineEdit, QComboBox, QDoubleSpinBox {
    background-color: #2b2b33;
    border: 1px solid #4c4c56;
    border-radius: 4px;
    padding: 4px 8px;
    color: #ffffff;
}
QComboBox QAbstractItemView {
    background-color: #2b2b33;
    color: #ffffff;
    border: 1px solid #4c4c56;
    selection-background-color: #3c3c44;
}
QLineEdit:focus, QComboBox:focus, QDoubleSpinBox:focus {
    border: 1px solid #00aaff;
}
QPushButton {
    background-color: #3c3c44;
    border: 1px solid #4c4c56;
    border-radius: 4px;
    padding: 6px 12px;
    color: #ffffff;
    font-weight: bold;
}
QPushButton:hover {
    background-color: #4c4c56;
}
QPushButton:pressed {
    background-color: #2b2b33;
}
QPushButton#btn_apply {
    background-color: #228b22;
    border-color: #2e8b57;
}
QPushButton#btn_apply:hover {
    background-color: #2ebd2e;
}
QPushButton#btn_restore {
    background-color: #b22222;
    border-color: #8b0000;
}
QPushButton#btn_restore:hover {
    background-color: #d32f2f;
}
QListWidget {
    background-color: #15151a;
    border: 1px solid #3c3c44;
    color: #e0e0e6;
    border-radius: 4px;
}
QListWidget::item:selected {
    background-color: #3a3a45;
    color: #ffffff;
}
QListWidget::item:hover {
    background-color: #2a2a35;
}
QLabel {
    color: #e0e0e6;
}
"""

# Config values lists from the guide
TRANSMISSION_PROFILES = [
    "Buggy", "Classic", "Electric_Hybrid", "Electric_Sportscar", "Electric_Supercar",
    "Electric_Hypercar", "Electric_HypercarALT", "Electric_HypercarALT2", "HotHatch",
    "HotRod", "Kei", "Muscle", "Offroad", "Race", "Rally", "Saloon", "SportsCar",
    "Supercar", "SUV", "TrackToy", "Truck", "TruckALT", "Van"
]

TURBO_PROFILES = [
    "Buggy", "Classic", "HotHatch", "HotRod", "Kei", "Muscle", "Offroad", "Race",
    "Rally", "Saloon", "SportsCar", "Supercar", "Hypercar", "SUV", "TrackToy"
]

SUPER_CSC_PROFILES = TURBO_PROFILES

SUPER_DSC_PROFILES = [
    "Buggy", "Classic", "HotHatch", "HotRod", "Kei", "Muscle", "Offroad", "Race",
    "Rally", "Saloon", "SportsCar", "Supercar", "Hypercar", "SUV", "TrackToy",
    "Truck", "Van", "MuscleALT"
]

BURBLES_OPTIONS = [
    "None",
    # V8
    "ModernV8Muscle", "ModernV8Offroad", "ModernV8SportsCar", "ModernV8SUV", "ModernV8Saloon",
    "ModernV8Supercar", "ModernV8TrackToy", "ModernV8Truck", "ModernV8Van", "ClassicV8Muscle",
    "ClassicV8Offroad", "ClassicV8Race", "ClassicV8Saloon", "ClassicV8SportsCar", "ClassicV8Supercar",
    "ClassicV8TrackToy", "VintageV8Classic", "VintageV8HotRod", "VintageV8Muscle", "VintageV8Offroad",
    "VintageV8Race",
    # V12
    "ModernV12Hypercar", "ModernV12Race", "ModernV12SUV", "ModernV12SportsCar", "ModernV12Supercar",
    "ModernV12TrackToy", "VintageV12Classic", "VintageV12Race",
    # V10
    "ModernV10Muscle", "ModernV10Race", "ModernV10Saloon", "ModernV10Supercar", "ModernV10TrackToy",
    "ClassicV10Muscle", "ClassicV10Race", "ClassicV10Rally",
    # V6
    "ModernV6Muscle", "ModernV6Offroad", "ModernV6Race", "ModernV6SUV", "ModernV6Saloon",
    "ModernV6SportsCar", "ModernV6Supercar", "ModernV6TrackToy", "ModernV6Truck", "ClassicV6Classic",
    "ClassicV6HotHatch", "ClassicV6Kei", "ClassicV6Muscle", "ClassicV6Rally", "ClassicV6SUV",
    "ClassicV6Saloon", "ClassicV6SportsCar", "ClassicV6Supercar", "ClassicV6TrackToy", "ClassicV6Van",
    "VintageV6Rally", "VintageV6SportsCar", "VintageV6TrackToy",
    # I6
    "ModernI6Race", "ModernI6SUV", "ModernI6Saloon", "ModernI6SportsCar", "ModernI6TrackToy",
    "ModernI6Truck", "ClassicI6Race", "ClassicI6Saloon", "ClassicI6SportsCar", "ClassicI6Supercar",
    "VintageI6Classic", "VintageI6Muscle", "VintageI6Race", "VintageI6SportsCar", "VintageI6Truck",
    # I5
    "ModernI5HotHatch", "ModernI5Saloon", "ModernI5SportsCar", "ModernI5TrackToy", "ClassicI5Rally",
    # I4
    "ModernI4Buggy", "ModernI4Classic", "ModernI4HotHatch", "ModernI4Offroad", "ModernI4Race",
    "ModernI4Rally", "ModernI4Saloon", "ModernI4SportsCar", "ModernI4Supercar", "ModernI4TrackToy",
    "ModernI4Van", "ClassicI4Classic", "ClassicI4HotHatch", "ClassicI4Kei", "ClassicI4Race",
    "ClassicI4Rally", "ClassicI4Saloon", "ClassicI4SportsCar", "VintageI4Classic", "VintageI4HotHatch",
    "VintageI4Rally", "VintageI4SportsCar",
    # I3/2/1
    "ModernI3Buggy", "ModernI3HotHatch", "ModernI3SportsCar", "ModernI3TrackToy", "ModernI2Offroad",
    "ModernI1Offroad", "ClassicI3Kei", "ClassicI3Race", "VintageI2Classic", "VintageI1Classic",
    # Flat
    "ModernF6Race", "ModernF6SportsCar", "ModernF6Supercar", "ModernF6TrackToy", "ClassicF6Race",
    "ClassicF6Rally", "ClassicF6SportsCar", "ClassicF6Supercar", "VintageF6Classic", "ModernF4Rally",
    "ModernF4Saloon", "ModernF4SportsCar", "ClassicF4Race", "ClassicF4Rally", "ClassicF4Saloon",
    "ClassicF4SportsCar", "ClassicF4Van", "VintageF4Buggy", "VintageF4Classic", "VintageF4Race",
    # Rotary
    "ModernRotary2SportsCar", "ModernRotary3Race", "ClassicRotary2SportsCar", "VintageRotary2Classic",
    "VintageRotary2SportsCar", "VintageRotary4Race"
]

BACKFIRE_ANTILAG_OPTIONS = [
    "None", "I1", "I2", "I3", "I4", "I5", "I6", "I8", "V4", "V6", "V8", "V10", "V12",
    "F2", "F4", "F6", "F12", "W12", "W16", "Rally", "Rotary", "Scallop", "E0"
]

BOV_OPTIONS = [
    "None", "Stock", "SportsCar", "Supercar", "Hypercar", "TrackToy", "Buggy", "Kei",
    "HotHatch", "SUV", "Saloon", "Truck", "Van", "Classic", "HotRod", "Muscle",
    "Offroad", "Race", "Rally", "Scallop", "JDM", "Diesel"
]

GEARCRACK_OPTIONS = [
    "None", "Manual", "DCT", "DSG", "Sequential", "GearCrack_Sequential", "Race"
]

LIMITER_OPTIONS = [
    "None", "Limiter_Vintage", "Limiter_Classic", "Limiter_Modern"
]

THROTTLE_OPTIONS = [
    "None", "ThrottleBody_Vintage", "ThrottleBody_Classic", "ThrottleBody_Modern"
]

ENGINE_BANK_OPTIONS = [
    "GS_ModularCar", "GS_ModularCar_ALT"
]

class ConfigManager:
    """Manages reading and writing app settings/overrides to ~/.local/share/ForzaHorizon6SoundMod/config.json"""
    def __init__(self):
        self.config_dir = os.path.join(os.path.expanduser("~"), ".local", "share", "ForzaHorizon6SoundMod")
        os.makedirs(self.config_dir, exist_ok=True)
        self.config_file = os.path.join(self.config_dir, "config.json")
        self.profiles_dir = os.path.join(self.config_dir, "profiles")
        os.makedirs(self.profiles_dir, exist_ok=True)
        
        self.data = self.load_config()
        self.data["profiles"] = self.load_profiles()
        
        # If no profiles at all (e.g. fresh start and migration didn't run because no old data), create Default
        if not self.data["profiles"]:
            self._create_default_profile()

    def _create_default_profile(self):
        default_data = {
            "car_overrides": {},
            "global_overrides": {},
            "synth_overrides": {},
            "misc_overrides": {},
            "reverb_overrides": {},
            "description": "Default profile",
            "created": datetime.datetime.now().isoformat(timespec="seconds")
        }
        self.save_profile("Default", default_data)
        self.data["profiles"] = {"Default": default_data}
        self.data["active_profile"] = "Default"
        self.save_config()

    def load_profiles(self):
        """Loads all profiles from the profiles directory."""
        profiles = {}
        for f in os.listdir(self.profiles_dir):
            if f.endswith(".json"):
                name = f[:-5]
                try:
                    with open(os.path.join(self.profiles_dir, f), "r") as pf:
                        profiles[name] = json.load(pf)
                except Exception as e:
                    print(f"Error loading profile {f}: {e}", file=sys.stderr)
        return profiles

    def load_config(self):
        default_data = {
            "game_path": "/home/mo/Schinken/Linux/Steam/steamapps/common/ForzaHorizon6",
            "global_overrides": {},
            "car_overrides": {},
            "synth_overrides": {},
            "misc_overrides": {},
            "reverb_overrides": {},
            "active_profile": "Default"
        }
        if os.path.exists(self.config_file):
            try:
                with open(self.config_file, "r") as f:
                    loaded = json.load(f)
                    
                    # Migration: If old monolithic 'profiles' dict exists, split to files
                    if "profiles" in loaded:
                        migrated = False
                        for p_name, p_data in loaded["profiles"].items():
                            try:
                                with open(os.path.join(self.profiles_dir, f"{p_name}.json"), "w") as pf:
                                    json.dump(p_data, pf, indent=2)
                                migrated = True
                            except Exception as e:
                                print(f"Error migrating profile {p_name}: {e}")
                        del loaded["profiles"] # Remove it from config.json
                        if migrated:
                            # Re-save config to drop the profiles block
                            with open(self.config_file, "w") as fw:
                                json.dump(loaded, fw, indent=2)
                    
                    # Merge default keys if they are missing
                    for k, v in default_data.items():
                        if k not in loaded:
                            loaded[k] = v
                    return loaded
            except Exception as e:
                print(f"Error loading config.json: {e}", file=sys.stderr)
        
        return default_data

    def save_config(self):
        try:
            # We don't save the dynamic "profiles" list into the main config.json
            save_data = {k: v for k, v in self.data.items() if k != "profiles"}
            with open(self.config_file, "w") as f:
                json.dump(save_data, f, indent=4)
        except Exception as e:
            print(f"Error saving config.json: {e}", file=sys.stderr)
            
    def save_profile(self, name, profile_data):
        """Saves a single profile to the profiles directory."""
        path = os.path.join(self.profiles_dir, f"{name}.json")
        try:
            with open(path, "w") as f:
                json.dump(profile_data, f, indent=2)
            if "profiles" in self.data:
                self.data["profiles"][name] = profile_data
        except Exception as e:
            print(f"Error saving profile {name}: {e}", file=sys.stderr)

    def delete_profile(self, name):
        """Deletes a profile from the profiles directory."""
        path = os.path.join(self.profiles_dir, f"{name}.json")
        if os.path.exists(path):
            try:
                os.remove(path)
                if "profiles" in self.data and name in self.data["profiles"]:
                    del self.data["profiles"][name]
            except Exception as e:
                print(f"Error deleting profile {name}: {e}", file=sys.stderr)

class SoundModifierEngine:
    """Manages file modifications, backups, and XML parsing/serialization."""
    def __init__(self, config_manager):
        self.cfg = config_manager
        # Cached list of cars dynamically loaded
        self.car_list = [] # List of dicts: {"id": "ACU_IntegraR_01", "file": "ACU_IntegraR_01-Engine.xml", "stock_values": {...}}
        # Dynamic options scraped from game files
        self.scraped_options = {
            "Engine": set(),
            "Exhaust": set(),
            "Intake": set(),
        }

    def get_audio_paths(self):
        game_path = self.cfg.data["game_path"]
        audio_path = os.path.join(game_path, "media", "Audio")
        backup_path = os.path.join(game_path, "media", "Audio_Backup")
        return audio_path, backup_path

    def get_misc_paths(self):
        game_path = self.cfg.data["game_path"]
        misc_dir = os.path.join(game_path, "media", "Cars", "_library")
        misc_file = os.path.join(misc_dir, "GlobalCarAttributes.xml")
        misc_backup = os.path.join(misc_dir, "GlobalCarAttributes_Backup.xml")
        return misc_file, misc_backup

    def check_backup_status(self):
        """Returns (has_backup, message)"""
        audio_path, backup_path = self.get_audio_paths()
        if not os.path.exists(audio_path):
            return False, "ForzaHorizon6/media/Audio folder not found at specified path!"
        if not os.path.exists(backup_path):
            return False, "Backup folder (Audio_Backup) not found."
        
        # Count xmls in backup
        xml_count = 0
        for root, dirs, files in os.walk(backup_path):
            xml_count += sum(1 for f in files if f.endswith(".xml"))
        
        if xml_count == 0:
            return False, "Backup folder is empty."
        return True, f"Backup complete ({xml_count} XML configuration files backed up)."

    def create_backup(self, progress_callback=None):
        """Replicates directories of media/Audio into media/Audio_Backup and copies .xml files ONLY."""
        audio_path, backup_path = self.get_audio_paths()
        if not os.path.exists(audio_path):
            raise FileNotFoundError(f"Audio folder not found at {audio_path}")

        # Clear existing backup if any
        if os.path.exists(backup_path):
            shutil.rmtree(backup_path)
        os.makedirs(backup_path, exist_ok=True)

        # Count total files first for progress
        total_xmls = 0
        for root, dirs, files in os.walk(audio_path):
            total_xmls += sum(1 for f in files if f.endswith(".xml"))

        copied_count = 0
        for root, dirs, files in os.walk(audio_path):
            rel_path = os.path.relpath(root, audio_path)
            dest_dir = backup_path if rel_path == "." else os.path.join(backup_path, rel_path)
            os.makedirs(dest_dir, exist_ok=True)
            for f in files:
                if f.endswith(".xml"):
                    src_file = os.path.join(root, f)
                    dest_file = os.path.join(dest_dir, f)
                    shutil.copy2(src_file, dest_file)
                    copied_count += 1
                    if progress_callback and total_xmls > 0:
                        progress_callback(int(copied_count * 100 / total_xmls))

        # Also backup GlobalCarAttributes.xml if it exists
        misc_file, misc_backup = self.get_misc_paths()
        if os.path.exists(misc_file) and not os.path.exists(misc_backup):
            shutil.copy2(misc_file, misc_backup)

    def restore_backup(self):
        """Restores original XMLs from backup."""
        audio_path, backup_path = self.get_audio_paths()
        if not os.path.exists(backup_path):
            raise FileNotFoundError("Backup folder not found! Cannot restore.")
        
        # Copy XMLs back to media/Audio
        for root, dirs, files in os.walk(backup_path):
            rel_path = os.path.relpath(root, backup_path)
            dest_dir = audio_path if rel_path == "." else os.path.join(audio_path, rel_path)
            os.makedirs(dest_dir, exist_ok=True)
            for f in files:
                if f.endswith(".xml"):
                    src_file = os.path.join(root, f)
                    dest_file = os.path.join(dest_dir, f)
                    shutil.copy2(src_file, dest_file)

        # Restore GlobalCarAttributes.xml
        misc_file, misc_backup = self.get_misc_paths()
        if os.path.exists(misc_backup):
            shutil.copy2(misc_backup, misc_file)

    def scan_cars(self):
        """Scans ModularCars folder in backup to load list of cars and scrape unique sound values."""
        self.car_list = []
        self.scraped_options = {
            "Engine": set(),
            "Exhaust": set(),
            "Intake": set(),
        }
        audio_path, backup_path = self.get_audio_paths()
        mc_backup_dir = os.path.join(backup_path, "ModularCars")
        if not os.path.exists(mc_backup_dir):
            return

        for filename in sorted(os.listdir(mc_backup_dir)):
            if filename.endswith("-Engine.xml"):
                car_id = filename[:-11] # strip '-Engine.xml'
                file_path = os.path.join(mc_backup_dir, filename)
                try:
                    tree = ET.parse(file_path)
                    root = tree.getroot()
                    stock_values = self.extract_values_from_xml(root)
                    
                    self.car_list.append({
                        "id": car_id,
                        "file": filename,
                        "stock_values": stock_values
                    })

                    # Collect unique engine notes to present in QComboBoxes
                    for level in ["Stock", "Street", "Sport", "Race"]:
                        for ch in ["Engine", "Exhaust", "Intake"]:
                            key = f"{ch}_{level}"
                            if key in stock_values:
                                val = stock_values[key]
                                if val:
                                    self.scraped_options[ch].add(val)
                except Exception as e:
                    print(f"Error reading car file {filename}: {e}", file=sys.stderr)

    def scan_synths(self):
        """Scans EngineSynth folder in backup to load list of synthesizers."""
        self.synth_list = []
        audio_path, backup_path = self.get_audio_paths()
        es_backup_dir = os.path.join(backup_path, "EngineSynth")
        if not os.path.exists(es_backup_dir):
            return

        for filename in sorted(os.listdir(es_backup_dir)):
            if filename.endswith(".xml"):
                self.synth_list.append(filename)

    def load_synth_values(self, filename):
        """Reads stock values of a synth XML file from backup."""
        audio_path, backup_path = self.get_audio_paths()
        src_file = os.path.join(backup_path, "EngineSynth", filename)
        values = {
            "Channel": {},
            "GearCrack": {}
        }
        if os.path.exists(src_file):
            try:
                tree = ET.parse(src_file)
                root = tree.getroot()
                channel = root.find(".//Channel")
                if channel is not None:
                    for attr, val in channel.attrib.items():
                        values["Channel"][attr] = val

                gearcrack = root.find(".//GearCrack")
                if gearcrack is not None:
                    for attr, val in gearcrack.attrib.items():
                        values["GearCrack"][attr] = val
            except Exception as e:
                print(f"Error reading synth values for {filename}: {e}", file=sys.stderr)
        return values

    def apply_synth_overrides_to_xml(self, root, overrides):
        # Apply Channel overrides
        if "Channel" in overrides:
            channel = root.find(".//Channel")
            if channel is not None:
                for attr, val in overrides["Channel"].items():
                    if val is not None:
                        channel.set(attr, str(val))

        # Apply GearCrack overrides
        if "GearCrack" in overrides:
            gearcrack = root.find(".//GearCrack")
            if gearcrack is not None:
                for attr, val in overrides["GearCrack"].items():
                    if val is not None:
                        gearcrack.set(attr, str(val))

    def extract_values_from_xml(self, root):
        values = {}
        
        # 1. Parameter RPMScalar
        param = root.find(".//GranularEngine/Parameter[@Name='RPMScalar']")
        if param is not None:
            for attr in ['Stock', 'Street', 'Sport', 'Race']:
                if attr in param.attrib:
                    values[f"RPMScalar_{attr}"] = param.attrib[attr]
                    
        # 2. Channels Engine, Exhaust, Intake
        for ch_name in ['Engine', 'Exhaust', 'Intake']:
            ch = root.find(f".//GranularEngine/Channel[@Name='{ch_name}']")
            if ch is not None:
                for attr in ['Stock', 'Street', 'Sport', 'Race']:
                    if attr in ch.attrib:
                        values[f"{ch_name}_{attr}"] = ch.attrib[attr]
                        
        # 3. Channels Turbo, SuperCSC, SuperDSC, Transmission
        for ch_name in ['Turbo', 'SuperCSC', 'SuperDSC', 'Transmission']:
            ch = root.find(f".//GranularEngine/Channel[@Name='{ch_name}']")
            if ch is not None:
                if 'Profile' in ch.attrib:
                    values[f"{ch_name}_Profile"] = ch.attrib['Profile']
                    
        # 4. Properties
        props = root.find(".//Properties")
        if props is not None:
            for prop in props.findall("Property"):
                name = prop.attrib.get('Name')
                val = prop.attrib.get('Value')
                if name and val is not None:
                    values[f"Prop_{name}"] = val
                    
        return values

    def apply_values_to_xml(self, root, overrides):
        # 1. Parameter RPMScalar
        param = root.find(".//GranularEngine/Parameter[@Name='RPMScalar']")
        if param is None:
            ge = root.find(".//GranularEngine")
            if ge is not None:
                param = ET.SubElement(ge, "Parameter", {"Name": "RPMScalar"})
        if param is not None:
            for attr in ['Stock', 'Street', 'Sport', 'Race']:
                key = f"RPMScalar_{attr}"
                if key in overrides and overrides[key] is not None:
                    param.set(attr, str(overrides[key]))
                    
        # 2. Channels Engine, Exhaust, Intake
        for ch_name in ['Engine', 'Exhaust', 'Intake']:
            ch = root.find(f".//GranularEngine/Channel[@Name='{ch_name}']")
            if ch is None:
                ge = root.find(".//GranularEngine")
                if ge is not None:
                    ch = ET.SubElement(ge, "Channel", {"Name": ch_name})
            if ch is not None:
                for attr in ['Stock', 'Street', 'Sport', 'Race']:
                    key = f"{ch_name}_{attr}"
                    if key in overrides and overrides[key] is not None:
                        ch.set(attr, str(overrides[key]))
                        
        # 3. Channels Turbo, SuperCSC, SuperDSC, Transmission
        for ch_name in ['Turbo', 'SuperCSC', 'SuperDSC', 'Transmission']:
            ch = root.find(f".//GranularEngine/Channel[@Name='{ch_name}']")
            if ch is None:
                ge = root.find(".//GranularEngine")
                if ge is not None:
                    ch = ET.SubElement(ge, "Channel", {"Name": ch_name})
            if ch is not None:
                key = f"{ch_name}_Profile"
                if key in overrides and overrides[key] is not None:
                    ch.set('Profile', str(overrides[key]))
                    
        # 4. Properties
        props = root.find(".//Properties")
        if props is None:
            props = ET.SubElement(root, "Properties")
            
        for key, val in overrides.items():
            if key.startswith("Prop_"):
                prop_name = key[5:] # remove 'Prop_'
                if val is None:
                    continue
                # Find existing property or create it
                prop = props.find(f"Property[@Name='{prop_name}']")
                if prop is None:
                    prop = ET.SubElement(props, "Property", {"Name": prop_name})
                prop.set('Value', str(val))

    def read_stock_misc_values(self):
        """Reads stock values of GlobalCarAttributes.xml from backup (or active if backup not made yet)."""
        misc_file, misc_backup = self.get_misc_paths()
        target_file = misc_backup if os.path.exists(misc_backup) else misc_file
        values = {"blur_rim_max_rpm": "350.0", "backfire_min_rand_time": "100.0", "backfire_max_rand_time": "350.0"}
        if os.path.exists(target_file):
            try:
                with open(target_file, "r", encoding="utf-8") as f:
                    content = f.read()
                import re
                blur_match = re.search(r'<BlurRim\s+[^>]*?MaxRPM="([^"]*?)"', content)
                if blur_match:
                    values["blur_rim_max_rpm"] = blur_match.group(1)
                
                min_match = re.search(r'<Effect\s+[^>]*?BackfireMinRandTime="([^"]*?)"', content)
                if min_match:
                    values["backfire_min_rand_time"] = min_match.group(1)
                
                max_match = re.search(r'<Effect\s+[^>]*?BackfireMaxRandTime="([^"]*?)"', content)
                if max_match:
                    values["backfire_max_rand_time"] = max_match.group(1)
            except Exception as e:
                print(f"Error reading misc stock values: {e}", file=sys.stderr)
        return values

    def apply_overrides(self, progress_callback=None):
        """Combines global and per-car overrides, applies to backup XMLs, and writes to media/Audio."""
        audio_path, backup_path = self.get_audio_paths()
        if not os.path.exists(backup_path):
            raise FileNotFoundError("Backup folder not found! Perform backup first.")

        # Re-scan backup directory to ensure structure matches
        mc_backup_dir = os.path.join(backup_path, "ModularCars")
        mc_active_dir = os.path.join(audio_path, "ModularCars")
        os.makedirs(mc_active_dir, exist_ok=True)

        global_overrides = self.cfg.data["global_overrides"]
        car_overrides_db = self.cfg.data["car_overrides"]

        car_files = [f for f in os.listdir(mc_backup_dir) if f.endswith("-Engine.xml")]
        total = len(car_files)
        
        # Write modified car engine files
        for idx, filename in enumerate(car_files):
            car_id = filename[:-11]
            src_file = os.path.join(mc_backup_dir, filename)
            dest_file = os.path.join(mc_active_dir, filename)
            
            # Combine overrides
            combined = {}
            # Start with global overrides
            for key, val in global_overrides.items():
                if val is not None:
                    combined[key] = val
            # Apply per-car overrides on top
            if car_id in car_overrides_db:
                for key, val in car_overrides_db[car_id].items():
                    if val is not None:
                        combined[key] = val

            try:
                tree = ET.parse(src_file)
                root = tree.getroot()
                if combined:
                    self.apply_values_to_xml(root, combined)
                
                # Write back
                tree.write(dest_file, encoding="utf-8", xml_declaration=True)
            except Exception as e:
                print(f"Failed to modify file {filename}: {e}", file=sys.stderr)

            if progress_callback and total > 0:
                progress_callback(int((idx + 1) * 80 / total)) # Use 80% for car files

        # Re-copy all OTHER XMLs in backup that aren't car engine files to audio_path
        # (e.g. Models XMLs, ModularCarConfig.xml, EngineSynth template XMLs)
        copy_idx = 0
        total_other = 0
        other_to_copy = []
        for root, dirs, files in os.walk(backup_path):
            rel_path = os.path.relpath(root, backup_path)
            if rel_path.startswith("ModularCars"):
                # ModularCars engine files are already written. We still copy other files in ModularCars, e.g. *-Model.xml
                for f in files:
                    if f.endswith(".xml") and not f.endswith("-Engine.xml"):
                        other_to_copy.append((os.path.join(root, f), os.path.join(audio_path, "ModularCars", f)))
            else:
                for f in files:
                    if f.endswith(".xml"):
                        other_to_copy.append((os.path.join(root, f), os.path.join(audio_path, rel_path, f)))
        
        total_other = len(other_to_copy)
        synth_overrides = self.cfg.data.get("synth_overrides", {})
        for idx, (src, dest) in enumerate(other_to_copy):
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            
            # Check if this is an EngineSynth XML and has active overrides (global category or specific)
            filename = os.path.basename(src)
            is_synth_file = "EngineSynth" in src and filename.endswith(".xml")
            
            if is_synth_file:
                # Determine category
                category = None
                if filename.endswith("_Eng.xml"):
                    category = "[Global Engine]"
                elif filename.endswith("_Exh.xml"):
                    category = "[Global Exhaust]"
                elif filename.endswith("_Int.xml"):
                    category = "[Global Intake]"
                elif filename.endswith("_Trn.xml"):
                    category = "[Global Transmission]"
                elif filename.endswith("_Tbo.xml"):
                    category = "[Global Turbo]"
                elif filename.endswith("_CSC.xml") or filename.endswith("_DSC.xml"):
                    category = "[Global Supercharger]"
                
                # Combine overrides
                combined_overrides = {}
                if category and category in synth_overrides:
                    for elem, attrs in synth_overrides[category].items():
                        combined_overrides[elem] = dict(attrs)
                        
                if filename in synth_overrides:
                    for elem, attrs in synth_overrides[filename].items():
                        if elem not in combined_overrides:
                            combined_overrides[elem] = {}
                        for attr, val in attrs.items():
                            combined_overrides[elem][attr] = val
                            
                if combined_overrides:
                    try:
                        tree = ET.parse(src)
                        root = tree.getroot()
                        self.apply_synth_overrides_to_xml(root, combined_overrides)
                        tree.write(dest, encoding="utf-8", xml_declaration=True)
                    except Exception as e:
                        print(f"Failed to modify synth file {filename}: {e}", file=sys.stderr)
                        shutil.copy2(src, dest)
                else:
                    shutil.copy2(src, dest)
            elif filename == "Reverb_Templates.xml":
                reverb_overrides = self.cfg.data.get("reverb_overrides", {})
                if reverb_overrides:
                    try:
                        tree = ET.parse(src)
                        root = tree.getroot()
                        for template in root.findall(".//Template"):
                            for attr in list(template.attrib.keys()):
                                multiplier = reverb_overrides.get(attr)
                                offset = reverb_overrides.get(attr + "_offset")
                                
                                if multiplier is not None or offset is not None:
                                    try:
                                        orig_str = template.attrib[attr]
                                        val = float(orig_str)
                                        
                                        new_val = val
                                        if multiplier is not None:
                                            new_val *= multiplier
                                        if offset is not None:
                                            new_val += offset
                                            
                                        if "." not in orig_str:
                                            template.set(attr, str(int(round(new_val))))
                                        else:
                                            template.set(attr, f"{new_val:.6f}")
                                    except ValueError:
                                        pass
                        tree.write(dest, encoding="utf-8", xml_declaration=True)
                    except Exception as e:
                        print(f"Failed to modify Reverb_Templates.xml: {e}", file=sys.stderr)
                        shutil.copy2(src, dest)
                else:
                    shutil.copy2(src, dest)
            else:
                shutil.copy2(src, dest)
                
            if progress_callback and total_other > 0:
                progress_callback(80 + int((idx + 1) * 15 / total_other)) # Use 15% for other copies

        # Apply miscellaneous overrides to GlobalCarAttributes.xml
        misc_file, misc_backup = self.get_misc_paths()
        if os.path.exists(misc_backup):
            try:
                with open(misc_backup, "r", encoding="utf-8") as f:
                    content = f.read()
                
                misc_overrides = self.cfg.data["misc_overrides"]
                import re
                
                # Apply BlurRim MaxRPM
                if "blur_rim_max_rpm" in misc_overrides:
                    pattern = r'(<BlurRim\s+[^>]*?MaxRPM=")([^"]*?)("[^>]*?>)'
                    content = re.sub(pattern, rf'\g<1>{misc_overrides["blur_rim_max_rpm"]}\g<3>', content)
                
                # Apply BackfireMinRandTime and BackfireMaxRandTime under Effect
                effect_match = re.search(r'<Effect\s+[^>]*?>', content)
                if effect_match:
                    tag = effect_match.group(0)
                    new_tag = tag
                    if "backfire_min_rand_time" in misc_overrides:
                        new_tag = re.sub(r'(BackfireMinRandTime=")([^"]*?)(")', rf'\g<1>{misc_overrides["backfire_min_rand_time"]}\g<3>', new_tag)
                    if "backfire_max_rand_time" in misc_overrides:
                        new_tag = re.sub(r'(BackfireMaxRandTime=")([^"]*?)(")', rf'\g<1>{misc_overrides["backfire_max_rand_time"]}\g<3>', new_tag)
                    content = content.replace(tag, new_tag)
                
                # Write to active path
                os.makedirs(os.path.dirname(misc_file), exist_ok=True)
                with open(misc_file, "w", encoding="utf-8") as f:
                    f.write(content)
            except Exception as e:
                print(f"Failed to modify GlobalCarAttributes.xml: {e}", file=sys.stderr)

        if progress_callback:
            progress_callback(100)

class MainWindow(QMainWindow):
    def __init__(self, cfg, engine):
        super().__init__()
        self.cfg = cfg
        self.engine = engine
        
        self.setWindowTitle("Forza Horizon 6 Sound Modifier")
        self.setMinimumSize(950, 700)
        self.setStyleSheet(DARK_STYLE)
        
        # Core UI initialization
        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        self.main_layout = QVBoxLayout(self.central_widget)
        
        # Header Area
        header_widget = QWidget()
        header_layout = QHBoxLayout(header_widget)
        header_layout.setContentsMargins(0, 0, 0, 10)
        
        title_label = QLabel("Forza Horizon 6 Sound Modifier")
        title_label.setFont(QFont("Arial", 16, QFont.Weight.Bold))
        header_layout.addWidget(title_label)
        
        header_layout.addStretch()
        
        self.btn_apply = QPushButton("Apply Mod Profile to Game")
        self.btn_apply.setObjectName("btn_apply")
        self.btn_apply.setFont(QFont("Arial", 10, QFont.Weight.Bold))
        self.btn_apply.setMinimumWidth(200)
        self.btn_apply.clicked.connect(self.action_apply_changes)
        header_layout.addWidget(self.btn_apply)
        
        self.main_layout.addWidget(header_widget)
        
        # Tabs
        self.tab_widget = QTabWidget()
        self.main_layout.addWidget(self.tab_widget)
        
        # Create Tabs
        self.init_setup_tab()
        self.init_car_tab()
        self.init_class_tab()
        self.init_misc_tab()
        self.init_reverb_tab()
        self.init_profiles_tab()
        
        # Footer / Status Bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        self.progress_bar.setTextVisible(False)
        self.progress_bar.setMaximumHeight(8)
        self.main_layout.addWidget(self.progress_bar)
        
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        self.status_bar.showMessage("Ready")
        
        # Load directories and display status
        self.update_backup_status()
        self.load_options_and_scan()

    def init_setup_tab(self):
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # Group Box: Folder Selection
        group_folder = QGroupBox("Folder Selection")
        layout_folder = QHBoxLayout(group_folder)
        
        layout_folder.addWidget(QLabel("Forza Horizon 6 Directory: "))
        self.edit_game_path = QLineEdit(self.cfg.data["game_path"])
        self.edit_game_path.textChanged.connect(self.game_path_changed)
        layout_folder.addWidget(self.edit_game_path)
        
        btn_browse = QPushButton("Browse...")
        btn_browse.clicked.connect(self.browse_game_path)
        layout_folder.addWidget(btn_browse)
        
        layout.addWidget(group_folder)
        
        # Group Box: Backup & Restore
        group_backup = QGroupBox("Backup & Restore Options")
        layout_backup = QVBoxLayout(group_backup)
        
        self.lbl_backup_status = QLabel("Checking backup status...")
        self.lbl_backup_status.setFont(QFont("Arial", 10, QFont.Weight.Bold))
        layout_backup.addWidget(self.lbl_backup_status)
        
        desc_backup = QLabel("To prevent game updates or corruptions from breaking your sound modifications, the tool reads configurations from a safe Backup folder ('media/Audio_Backup') and applies modifications to the active game folder. This backup copies only configuration files (.xml), avoiding gigabytes of audio banks.")
        desc_backup.setWordWrap(True)
        desc_backup.setStyleSheet("color: #a0a0aa;")
        layout_backup.addWidget(desc_backup)
        
        btn_layout = QHBoxLayout()
        self.btn_create_backup = QPushButton("Create/Refresh Backup")
        self.btn_create_backup.clicked.connect(self.action_create_backup)
        btn_layout.addWidget(self.btn_create_backup)
        
        self.btn_restore = QPushButton("Restore Original Stock Files")
        self.btn_restore.setObjectName("btn_restore")
        self.btn_restore.clicked.connect(self.action_restore)
        btn_layout.addWidget(self.btn_restore)
        
        layout_backup.addLayout(btn_layout)
        
        layout.addWidget(group_backup)
        
        # Instructions Box
        group_inst = QGroupBox("Usage Instructions")
        layout_inst = QVBoxLayout(group_inst)
        lbl_inst = QLabel(
            "1. Select your ForzaHorizon6 game path above.\n"
            "2. Click 'Create/Refresh Backup' to safe-keep original files (required before applying edits).\n"
            "3. Customize sound profiles globally or on a per-car basis under the respective tabs.\n"
            "4. Modify miscellaneous settings like backfire duration and BlurRim rotational speed limit.\n"
            "5. Click the green 'Apply Mod Profile to Game' button at the top-right to write overrides to the game files."
        )
        lbl_inst.setWordWrap(True)
        layout_inst.addWidget(lbl_inst)
        layout.addWidget(group_inst)
        
        layout.addStretch()
        self.tab_widget.addTab(tab, "Setup / Path")

    def build_global_config_widget(self):
        self.scroll_global_config = QScrollArea()
        self.scroll_global_config.setWidgetResizable(True)
        self.scroll_global_config.setFrameShape(QFrame.Shape.NoFrame if hasattr(QFrame, "Shape") else QFrame.NoFrame)
        self.scroll_global_config.setVisible(False)
        
        content = QWidget()
        self.scroll_global_config.setWidget(content)
        self.global_form_layout = QVBoxLayout(content)
        self.global_form_layout.setContentsMargins(15, 10, 15, 10)
        
        # We will hold widgets in dictionaries to check state easily
        self.global_widgets = {} # parameter_key -> {"cb": QCheckBox, "control": QWidget}
        
        # 1. RPM & Core Channels Group
        grp_core = QGroupBox("Core Synthesizer & RPM Channels (Global)")
        layout_core = QGridLayout(grp_core)
        
        # Columns Headers: Enable/Param, Stock, Street, Sport, Race
        layout_core.addWidget(QLabel("Parameter"), 0, 0)
        layout_core.addWidget(QLabel("Stock"), 0, 1)
        layout_core.addWidget(QLabel("Street"), 0, 2)
        layout_core.addWidget(QLabel("Sport"), 0, 3)
        layout_core.addWidget(QLabel("Race"), 0, 4)
        
        # RPMScalar row
        self.add_global_channel_row(layout_core, "RPMScalar", 1, is_float=True)
        self.add_global_channel_row(layout_core, "Engine", 2)
        self.add_global_channel_row(layout_core, "Exhaust", 3)
        self.add_global_channel_row(layout_core, "Intake", 4)
        
        self.global_form_layout.addWidget(grp_core)
        
        # 2. Whining Profiles Group
        grp_whine = QGroupBox("Induction & Gear Whine Profiles (Global)")
        layout_whine = QFormLayout(grp_whine)
        
        self.add_global_profile_row(layout_whine, "Turbo", TURBO_PROFILES)
        self.add_global_profile_row(layout_whine, "SuperCSC", SUPER_CSC_PROFILES)
        self.add_global_profile_row(layout_whine, "SuperDSC", SUPER_DSC_PROFILES)
        self.add_global_profile_row(layout_whine, "Transmission", TRANSMISSION_PROFILES)
        
        self.global_form_layout.addWidget(grp_whine)
        
        # 3. Acoustic Properties
        grp_props = QGroupBox("Acoustic Sound Properties (Global)")
        layout_props = QFormLayout(grp_props)
        
        self.add_global_prop_row(layout_props, "EngineBank", ENGINE_BANK_OPTIONS)
        self.add_global_prop_row(layout_props, "Burbles", BURBLES_OPTIONS)
        self.add_global_prop_row(layout_props, "Backfire", BACKFIRE_ANTILAG_OPTIONS)
        self.add_global_prop_row(layout_props, "AntiLag", BACKFIRE_ANTILAG_OPTIONS)
        self.add_global_prop_row(layout_props, "TurboBOV", BOV_OPTIONS)
        self.add_global_prop_row(layout_props, "CentrifugalBOV", BOV_OPTIONS)
        self.add_global_prop_row(layout_props, "ThrottleBody", THROTTLE_OPTIONS)
        self.add_global_prop_row(layout_props, "Limiter", LIMITER_OPTIONS)
        self.add_global_prop_row(layout_props, "GearCrack", GEARCRACK_OPTIONS)
        
        self.global_form_layout.addWidget(grp_props)

    def add_global_channel_row(self, grid_layout, param_name, row_idx, is_float=False):
        """Adds a multi-level stock/street/sport/race override row."""
        cb_master = QCheckBox(param_name)
        grid_layout.addWidget(cb_master, row_idx, 0)
        
        controls = []
        levels = ["Stock", "Street", "Sport", "Race"]
        
        for col, level in enumerate(levels):
            key = f"{param_name}_{level}"
            if is_float:
                edit = QLineEdit()
                edit.setPlaceholderText("Default")
                val = QDoubleValidator(0.0, 5.0, 4, edit)
                val.setLocale(QLocale("C"))
                edit.setValidator(val)
                control = edit
            else:
                combo = QComboBox()
                combo.setEditable(True)
                combo.addItem("") # None/Default option
                control = combo
                
            control.setEnabled(False)
            grid_layout.addWidget(control, row_idx, col + 1)
            controls.append(control)
            
            # Register in widgets dictionary
            self.global_widgets[key] = {"cb": cb_master, "control": control}
            
        # Wire up master checkbox to toggle enabled state of inputs
        def on_toggle(checked):
            for c in controls:
                c.setEnabled(checked)
            self.save_global_from_ui()
            self.validate_selected_car_highlights()
            
        cb_master.toggled.connect(on_toggle)
        for ctrl in controls:
            if isinstance(ctrl, QLineEdit):
                ctrl.textChanged.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))
            elif isinstance(ctrl, QComboBox):
                ctrl.currentTextChanged.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))

    def add_global_profile_row(self, form_layout, profile_name, choices):
        """Adds a profile override row."""
        h_layout = QHBoxLayout()
        cb = QCheckBox("Override")
        h_layout.addWidget(cb)
        
        combo = QComboBox()
        combo.setEditable(True)
        combo.addItem("")
        combo.addItems(choices)
        combo.setEnabled(False)
        h_layout.addWidget(combo)
        
        form_layout.addRow(QLabel(profile_name), h_layout)
        
        key = f"{profile_name}_Profile"
        self.global_widgets[key] = {"cb": cb, "control": combo}
        cb.toggled.connect(combo.setEnabled)
        cb.toggled.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))
        combo.currentTextChanged.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))

    def add_global_prop_row(self, form_layout, prop_name, choices):
        """Adds a property override row."""
        h_layout = QHBoxLayout()
        cb = QCheckBox("Override")
        h_layout.addWidget(cb)
        
        combo = QComboBox()
        combo.setEditable(True)
        combo.addItem("")
        combo.addItems(choices)
        combo.setEnabled(False)
        h_layout.addWidget(combo)
        
        form_layout.addRow(QLabel(prop_name), h_layout)
        
        key = f"Prop_{prop_name}"
        self.global_widgets[key] = {"cb": cb, "control": combo}
        cb.toggled.connect(combo.setEnabled)
        cb.toggled.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))
        combo.currentTextChanged.connect(lambda: (self.save_global_from_ui(), self.validate_selected_car_highlights()))

    def validate_selected_car_highlights(self):
        current_car = self.list_cars.currentItem()
        if current_car and current_car.text() != "[Global Overrides]":
            for key in self.car_widgets:
                self.validate_car_highlights(key)

    def init_car_tab(self):
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        splitter = QSplitter(Qt.Orientation.Horizontal if hasattr(Qt, "Orientation") else Qt.Horizontal)
        layout.addWidget(splitter)
        
        # Left Side Pane: List & Search
        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        left_layout.setContentsMargins(0, 0, 0, 0)
        
        self.search_cars = QLineEdit()
        self.search_cars.setPlaceholderText("Search Cars (e.g. Golf, ACU)...")
        self.search_cars.textChanged.connect(self.filter_car_list)
        left_layout.addWidget(self.search_cars)
        
        self.list_cars = QListWidget()
        self.list_cars.currentItemChanged.connect(self.car_selection_changed)
        left_layout.addWidget(self.list_cars)
        
        splitter.addWidget(left_widget)
        
        # Right Side Pane: Editing Controls
        right_widget = QWidget()
        self.right_layout = QVBoxLayout(right_widget)
        self.right_layout.setContentsMargins(0, 0, 0, 0)
        
        # Empty/Welcome View
        self.lbl_per_car_welcome = QLabel("Select a vehicle or [Global Overrides] from the list to modify sound overrides.")
        self.lbl_per_car_welcome.setAlignment(get_align_center())
        self.lbl_per_car_welcome.setStyleSheet("color: #a0a0aa; font-size: 11pt;")
        self.right_layout.addWidget(self.lbl_per_car_welcome)
        
        # Global Configuration Scroll Area
        self.build_global_config_widget()
        self.right_layout.addWidget(self.scroll_global_config)
        
        # Configuration Scroll Area
        self.scroll_car_config = QScrollArea()
        self.scroll_car_config.setWidgetResizable(True)
        self.scroll_car_config.setVisible(False)
        self.right_layout.addWidget(self.scroll_car_config)
        
        self.car_config_container = QWidget()
        self.scroll_car_config.setWidget(self.car_config_container)
        self.car_form_layout = QVBoxLayout(self.car_config_container)
        
        # Header for the selected car
        self.car_header_layout = QHBoxLayout()
        self.lbl_selected_car_id = QLabel("Car ID")
        self.lbl_selected_car_id.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        self.car_header_layout.addWidget(self.lbl_selected_car_id)
        
        self.car_header_layout.addStretch()
        
        btn_clear_car = QPushButton("Clear Overrides For This Car")
        btn_clear_car.clicked.connect(self.action_clear_car_overrides)
        self.car_header_layout.addWidget(btn_clear_car)
        
        self.car_form_layout.addLayout(self.car_header_layout)
        
        # Dict to store UI widgets for per-car modifications
        self.car_widgets = {}
        
        # Group 1: RPM & Core Channels Grid
        grp_core = QGroupBox("Core Sound Channels")
        layout_core = QGridLayout(grp_core)
        layout_core.setColumnStretch(0, 3) # Name
        layout_core.setColumnStretch(1, 4) # Stock label
        layout_core.setColumnStretch(2, 1) # Override checkbox
        layout_core.setColumnStretch(3, 4) # Override Input
        layout_core.setColumnStretch(4, 1) # Navigation Link Button
        
        layout_core.addWidget(QLabel("Parameter"), 0, 0)
        layout_core.addWidget(QLabel("Original Stock Value"), 0, 1)
        layout_core.addWidget(QLabel("Override?"), 0, 2)
        layout_core.addWidget(QLabel("New Override Value"), 0, 3)
        layout_core.addWidget(QLabel("Link"), 0, 4)
        
        row = 1
        # Add core channels
        for param in ["RPMScalar", "Engine", "Exhaust", "Intake"]:
            for level in ["Stock", "Street", "Sport", "Race"]:
                key = f"{param}_{level}"
                is_float = (param == "RPMScalar")
                self.add_car_row_grid(layout_core, key, f"{param} ({level})", row, is_float=is_float)
                row += 1
                
        self.car_form_layout.addWidget(grp_core)
        
        # Group 2: Whines Form
        grp_whine = QGroupBox("Induction & Gear Whine Profiles")
        layout_whine = QFormLayout(grp_whine)
        self.add_car_row_form(layout_whine, "Turbo_Profile", "Turbo", TURBO_PROFILES)
        self.add_car_row_form(layout_whine, "SuperCSC_Profile", "SuperCSC", SUPER_CSC_PROFILES)
        self.add_car_row_form(layout_whine, "SuperDSC_Profile", "SuperDSC", SUPER_DSC_PROFILES)
        self.add_car_row_form(layout_whine, "Transmission_Profile", "Transmission", TRANSMISSION_PROFILES)
        self.car_form_layout.addWidget(grp_whine)
        
        # Group 3: Properties Form
        grp_props = QGroupBox("Acoustic Sound Properties")
        layout_props = QFormLayout(grp_props)
        self.add_car_row_form(layout_props, "Prop_EngineBank", "EngineBank", ENGINE_BANK_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_Burbles", "Burbles", BURBLES_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_Backfire", "Backfire", BACKFIRE_ANTILAG_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_AntiLag", "AntiLag", BACKFIRE_ANTILAG_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_TurboBOV", "TurboBOV", BOV_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_CentrifugalBOV", "CentrifugalBOV", BOV_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_ThrottleBody", "ThrottleBody", THROTTLE_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_Limiter", "Limiter", LIMITER_OPTIONS)
        self.add_car_row_form(layout_props, "Prop_GearCrack", "GearCrack", GEARCRACK_OPTIONS)
        self.car_form_layout.addWidget(grp_props)
        
        splitter.addWidget(right_widget)
        
        # Set splitter sizes (250px left, remainder right)
        splitter.setSizes([250, 700])
        
        self.tab_widget.addTab(tab, "Car Overrides")

    def add_car_row_grid(self, grid_layout, key, display_name, row, is_float=False):
        """Adds a row for RPMScalar/Engine/Exhaust/Intake under the configuration grid."""
        lbl_name = QLabel(display_name)
        grid_layout.addWidget(lbl_name, row, 0)
        
        lbl_stock = QLabel("Stock: ...")
        lbl_stock.setStyleSheet("color: #a0a0aa;")
        grid_layout.addWidget(lbl_stock, row, 1)
        
        cb = QCheckBox()
        grid_layout.addWidget(cb, row, 2)
        
        if is_float:
            edit = QLineEdit()
            val = QDoubleValidator(0.0, 5.0, 4, edit)
            val.setLocale(QLocale("C"))
            edit.setValidator(val)
            control = edit
        else:
            control = QComboBox()
            control.setEditable(True)
            control.addItem("")
            
        control.setEnabled(False)
        grid_layout.addWidget(control, row, 3)
        
        btn_link = None
        if not is_float:
            btn_link = QPushButton("🔗")
            btn_link.setFixedWidth(35)
            btn_link.setToolTip("Go to Class Override")
            btn_link.setStyleSheet("padding: 2px; background-color: #2b2b33; border: 1px solid #4c4c56;")
            grid_layout.addWidget(btn_link, row, 4)
            btn_link.clicked.connect(lambda: self.go_to_synth_class(key))

        self.car_widgets[key] = {
            "cb": cb,
            "control": control,
            "lbl_stock": lbl_stock,
            "lbl_name": lbl_name,
            "btn_link": btn_link
        }
        
        # Trigger validation highlights when values change
        cb.toggled.connect(control.setEnabled)
        cb.toggled.connect(lambda: self.validate_car_highlights(key))
        if is_float:
            control.textChanged.connect(lambda: self.validate_car_highlights(key))
        else:
            control.currentTextChanged.connect(lambda: self.validate_car_highlights(key))

    def go_to_synth_class(self, key):
        current_car = self.list_cars.currentItem()
        if not current_car:
            return
        car_id = current_car.text()
        val = None
        
        if car_id == "[Global Overrides]":
            # Try global override first, then fall through to first available car's stock value
            w_info = self.global_widgets.get(key)
            if w_info and w_info["cb"].isChecked():
                ctrl = w_info["control"]
                val = ctrl.text() if isinstance(ctrl, QLineEdit) else ctrl.currentText()
            # Fallback: peek at first car's stock value for this key
            if not val or not val.strip():
                for car in self.engine.car_list:
                    sv = car["stock_values"].get(key)
                    if sv and sv.strip():
                        val = sv
                        break
        else:
            car_info = next((c for c in self.engine.car_list if c["id"] == car_id), None)
            if not car_info:
                return
            stock_val = car_info["stock_values"].get(key)
            w_info = self.car_widgets.get(key)
            if not w_info:
                return
            if w_info["cb"].isChecked():
                ctrl = w_info["control"]
                val = ctrl.text() if isinstance(ctrl, QLineEdit) else ctrl.currentText()
            else:
                global_w = self.global_widgets.get(key)
                if global_w and global_w["cb"].isChecked():
                    global_ctrl = global_w["control"]
                    val = global_ctrl.text() if isinstance(global_ctrl, QLineEdit) else global_ctrl.currentText()
                else:
                    val = stock_val
                    
        if val and val.strip():
            filename = f"{val.strip()}.xml"
            # Find Class Overrides tab by label to be index-independent
            for i in range(self.tab_widget.count()):
                if self.tab_widget.tabText(i) == "Class Overrides":
                    self.tab_widget.setCurrentIndex(i)
                    break
            self.search_synths.setText("")
            self.populate_synth_list()
            items = self.list_synths.findItems(filename, Qt.MatchFlag.MatchExactly if hasattr(Qt, "MatchFlag") else Qt.MatchExactly)
            if items:
                self.list_synths.setCurrentItem(items[0])
            else:
                QMessageBox.warning(self, "Not Found", f"Synthesizer config file '{filename}' was not found in backup/EngineSynth.")


    def add_car_row_form(self, form_layout, key, display_name, choices):
        """Adds a profile/property row under the car configuration form layout."""
        row_widget = QWidget()
        row_layout = QHBoxLayout(row_widget)
        row_layout.setContentsMargins(0, 0, 0, 0)
        
        lbl_stock = QLabel("Stock: ...")
        lbl_stock.setStyleSheet("color: #a0a0aa;")
        row_layout.addWidget(lbl_stock, 1)
        
        cb = QCheckBox("Override")
        row_layout.addWidget(cb)
        
        combo = QComboBox()
        combo.setEditable(True)
        combo.addItem("")
        combo.addItems(choices)
        combo.setEnabled(False)
        row_layout.addWidget(combo, 2)
        
        form_layout.addRow(QLabel(display_name), row_widget)
        
        self.car_widgets[key] = {
            "cb": cb,
            "control": combo,
            "lbl_stock": lbl_stock,
            "lbl_name": None # Form layout manages label
        }
        
        cb.toggled.connect(combo.setEnabled)
        cb.toggled.connect(lambda: self.validate_car_highlights(key))
        combo.currentTextChanged.connect(lambda: self.validate_car_highlights(key))

    def init_class_tab(self):
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        splitter = QSplitter(Qt.Orientation.Horizontal if hasattr(Qt, "Orientation") else Qt.Horizontal)
        layout.addWidget(splitter)
        
        # Left Side Pane: List & Search
        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        left_layout.setContentsMargins(0, 0, 0, 0)
        
        self.search_synths = QLineEdit()
        self.search_synths.setPlaceholderText("Search Class Synthesizers...")
        self.search_synths.textChanged.connect(self.filter_synth_list)
        left_layout.addWidget(self.search_synths)
        
        self.list_synths = QListWidget()
        self.list_synths.currentItemChanged.connect(self.synth_selection_changed)
        left_layout.addWidget(self.list_synths)
        
        splitter.addWidget(left_widget)
        
        # Right Side Pane: Editing Controls
        right_widget = QWidget()
        self.synth_right_layout = QVBoxLayout(right_widget)
        self.synth_right_layout.setContentsMargins(0, 0, 0, 0)
        
        # Empty/Welcome View
        self.lbl_synth_welcome = QLabel("Select a synthesizer file from the list to modify its parameters.")
        self.lbl_synth_welcome.setAlignment(get_align_center())
        self.lbl_synth_welcome.setStyleSheet("color: #a0a0aa; font-size: 11pt;")
        self.synth_right_layout.addWidget(self.lbl_synth_welcome)
        
        # Configuration Scroll Area
        self.scroll_synth_config = QScrollArea()
        self.scroll_synth_config.setWidgetResizable(True)
        self.scroll_synth_config.setVisible(False)
        self.synth_right_layout.addWidget(self.scroll_synth_config)
        
        self.synth_config_container = QWidget()
        self.scroll_synth_config.setWidget(self.synth_config_container)
        self.synth_form_layout = QVBoxLayout(self.synth_config_container)
        
        # Header for the selected synth
        self.synth_header_layout = QHBoxLayout()
        self.lbl_selected_synth_id = QLabel("Synth File")
        self.lbl_selected_synth_id.setFont(QFont("Arial", 12, QFont.Weight.Bold))
        self.synth_header_layout.addWidget(self.lbl_selected_synth_id)
        
        self.synth_header_layout.addStretch()
        
        btn_clear_synth = QPushButton("Clear Overrides For This Synth")
        btn_clear_synth.clicked.connect(self.action_clear_synth_overrides)
        self.synth_header_layout.addWidget(btn_clear_synth)
        
        self.synth_form_layout.addLayout(self.synth_header_layout)
        
        # Context description label (shown for globals and specific files)
        self.lbl_synth_description = QLabel("")
        self.lbl_synth_description.setWordWrap(True)
        self.lbl_synth_description.setStyleSheet(
            "color: #a0c8ff; background-color: #1e2a3a; border: 1px solid #2a4a6a; "
            "border-radius: 4px; padding: 6px 10px; margin-bottom: 4px;"
        )
        self.lbl_synth_description.setVisible(False)
        self.synth_form_layout.addWidget(self.lbl_synth_description)
        
        self.synth_widgets = {}
        
        # Groups
        self.grp_synth_channel = QGroupBox("Channel Volume & RPM Settings")
        self.layout_synth_channel = QFormLayout(self.grp_synth_channel)
        self.synth_form_layout.addWidget(self.grp_synth_channel)
        
        self.grp_synth_gearcrack = QGroupBox("Gear Crack Settings")
        self.layout_synth_gearcrack = QFormLayout(self.grp_synth_gearcrack)
        self.synth_form_layout.addWidget(self.grp_synth_gearcrack)
        
        self.grp_synth_advanced = QGroupBox("Advanced Synthesizer Attributes")
        self.layout_synth_advanced = QFormLayout(self.grp_synth_advanced)
        self.synth_form_layout.addWidget(self.grp_synth_advanced)
        
        splitter.addWidget(right_widget)
        splitter.setSizes([250, 700])
        
        self.tab_widget.addTab(tab, "Class Overrides")

    def populate_synth_list(self):
        self.list_synths.clear()
        
        # Add special global entries at the top (always visible, not filtered)
        for g in ["[Global Engine]", "[Global Exhaust]", "[Global Intake]",
                  "[Global Transmission]", "[Global Turbo]", "[Global Supercharger]"]:
            self.list_synths.addItem(g)
        
        filter_text = self.search_synths.text().lower()
        if not hasattr(self.engine, "synth_list"):
            return
        for filename in self.engine.synth_list:
            if filter_text in filename.lower():
                self.list_synths.addItem(filename)

    def filter_synth_list(self):
        self.populate_synth_list()

    def synth_selection_changed(self, current_item):
        if not current_item:
            self.lbl_synth_welcome.setVisible(True)
            self.scroll_synth_config.setVisible(False)
            return
            
        self.lbl_synth_welcome.setVisible(False)
        self.scroll_synth_config.setVisible(True)
        
        filename = current_item.text()
        self.lbl_selected_synth_id.setText(filename)
        self.build_synth_controls(filename)

    def clear_form_layout(self, layout):
        while layout.count() > 0:
            item = layout.takeAt(0)
            widget = item.widget()
            if widget is not None:
                widget.deleteLater()

    # Map of global category names -> (label description, suffix list for matching)
    GLOBAL_CATEGORY_INFO = {
        "[Global Engine]": (
            "Engine synthesizer files (*_Eng.xml)",
            "Sets Channel volume and RPM range for ALL engine synthesizer files. "
            "Specific file overrides will take precedence over these global defaults."
        ),
        "[Global Exhaust]": (
            "Exhaust synthesizer files (*_Exh.xml)",
            "Sets Channel volume and RPM range for ALL exhaust synthesizer files. "
            "Specific file overrides will take precedence over these global defaults."
        ),
        "[Global Intake]": (
            "Intake synthesizer files (*_Int.xml)",
            "Sets Channel volume and RPM range for ALL intake synthesizer files. "
            "Specific file overrides will take precedence over these global defaults."
        ),
        "[Global Transmission]": (
            "Transmission synthesizer files (*_Trn.xml)",
            "Sets Channel volume and RPM range for ALL transmission synthesizer files. "
            "Specific file overrides will take precedence over these global defaults."
        ),
        "[Global Turbo]": (
            "Turbocharger synthesizer files (*_Tbo.xml)",
            "Sets Channel volume and RPM range for ALL turbocharger synthesizer files. "
            "Specific file overrides will take precedence over these global defaults."
        ),
        "[Global Supercharger]": (
            "Supercharger synthesizer files (*_CSC.xml, *_DSC.xml)",
            "Sets Channel volume and RPM range for ALL supercharger synthesizer files "
            "(centrifugal and displacement). Specific file overrides will take precedence."
        ),
    }

    def build_synth_controls(self, filename):
        self.clear_form_layout(self.layout_synth_channel)
        self.clear_form_layout(self.layout_synth_gearcrack)
        self.clear_form_layout(self.layout_synth_advanced)
        
        self.synth_widgets = {}
        
        is_global = filename.startswith("[Global ")
        
        if is_global:
            self.grp_synth_gearcrack.setVisible(True)
            self.grp_synth_advanced.setVisible(False)

            # Update group box titles to reflect global context
            cat_info = self.GLOBAL_CATEGORY_INFO.get(filename, (filename, ""))
            cat_label, cat_desc = cat_info
            self.grp_synth_channel.setTitle(f"Channel Volume & RPM  \u2014  {cat_label}")
            self.grp_synth_gearcrack.setTitle(f"Gear Crack Settings  \u2014  {cat_label}")

            # Show description
            self.lbl_synth_description.setText(
                f"\u2139\ufe0f  {cat_desc}"
            )
            self.lbl_synth_description.setVisible(True)
            
            overrides = self.cfg.data.get("synth_overrides", {}).get(filename, {})
            channel_overrides = overrides.get("Channel", {})
            gearcrack_overrides = overrides.get("GearCrack", {})
            
            # Channel rows (no stock value for globals)
            self.add_synth_control_row(self.layout_synth_channel, "Channel", "MasterVolume", None, channel_overrides.get("MasterVolume"), filename)
            self.add_synth_control_row(self.layout_synth_channel, "Channel", "MinRPM", None, channel_overrides.get("MinRPM"), filename)
            self.add_synth_control_row(self.layout_synth_channel, "Channel", "MaxRPM", None, channel_overrides.get("MaxRPM"), filename)
            
            # GearCrack rows
            self.add_synth_control_row(self.layout_synth_gearcrack, "GearCrack", "Volume", None, gearcrack_overrides.get("Volume"), filename)
            self.add_synth_control_row(self.layout_synth_gearcrack, "GearCrack", "MinRPM", None, gearcrack_overrides.get("MinRPM"), filename)
            
        else:
            # Restore generic titles for specific file selection
            self.grp_synth_channel.setTitle("Channel Volume & RPM Settings")
            self.grp_synth_gearcrack.setTitle("Gear Crack Settings")
            self.lbl_synth_description.setVisible(False)

            stock_values = self.engine.load_synth_values(filename)
            overrides = self.cfg.data.get("synth_overrides", {}).get(filename, {})
            
            core_channel_attrs = ["MasterVolume", "MinRPM", "MaxRPM"]
            core_gearcrack_attrs = ["Volume", "MinRPM"]
            
            # 1. Channel Controls
            channel_stock = stock_values.get("Channel", {})
            channel_overrides = overrides.get("Channel", {})
            
            for attr in core_channel_attrs:
                if attr in channel_stock:
                    self.add_synth_control_row(
                        self.layout_synth_channel,
                        "Channel",
                        attr,
                        channel_stock[attr],
                        channel_overrides.get(attr),
                        filename
                    )
                    
            # 2. GearCrack Controls
            gearcrack_stock = stock_values.get("GearCrack", {})
            gearcrack_overrides = overrides.get("GearCrack", {})
            
            if gearcrack_stock:
                self.grp_synth_gearcrack.setVisible(True)
                for attr in core_gearcrack_attrs:
                    if attr in gearcrack_stock:
                        self.add_synth_control_row(
                            self.layout_synth_gearcrack,
                            "GearCrack",
                            attr,
                            gearcrack_stock[attr],
                            gearcrack_overrides.get(attr),
                            filename
                        )
            else:
                self.grp_synth_gearcrack.setVisible(False)
                
            # 3. Advanced Controls
            has_advanced = False
            for attr, val in channel_stock.items():
                if attr not in core_channel_attrs:
                    self.add_synth_control_row(
                        self.layout_synth_advanced,
                        "Channel",
                        attr,
                        val,
                        channel_overrides.get(attr),
                        filename
                    )
                    has_advanced = True
                    
            for attr, val in gearcrack_stock.items():
                if attr not in core_gearcrack_attrs:
                    self.add_synth_control_row(
                        self.layout_synth_advanced,
                        "GearCrack",
                        attr,
                        val,
                        gearcrack_overrides.get(attr),
                        filename
                    )
                    has_advanced = True
                    
            self.grp_synth_advanced.setVisible(has_advanced)

    def add_synth_control_row(self, form_layout, element, attr, stock_val, override_val, filename):
        """stock_val=None means this is a global entry (no baseline file)."""
        is_global_entry = (stock_val is None)
        row_widget = QWidget()
        row_layout = QHBoxLayout(row_widget)
        row_layout.setContentsMargins(0, 0, 0, 0)
        
        if is_global_entry:
            lbl_stock = QLabel("Global default")
            lbl_stock.setStyleSheet("color: #6888aa; font-style: italic;")
        else:
            lbl_stock = QLabel(f"Stock: {stock_val}")
            lbl_stock.setStyleSheet("color: #a0a0aa;")
        row_layout.addWidget(lbl_stock, 1)
        
        cb = QCheckBox("Override")
        row_layout.addWidget(cb)
        
        # Treat stock_val as empty string for type-detection when None (global)
        _stock_for_type = stock_val if stock_val is not None else ""
        is_bool = _stock_for_type.lower() in ["true", "false"]
        if is_bool:
            control = QComboBox()
            control.addItems(["True", "False"])
        else:
            control = QLineEdit()
            is_numeric = False
            try:
                float(_stock_for_type)
                is_numeric = True
            except ValueError:
                if attr in ["MasterVolume", "MinRPM", "MaxRPM", "Volume"]:
                    is_numeric = True
                    
            if is_numeric:
                val_obj = QDoubleValidator(0.0, 1000000.0, 4, control)
                val_obj.setLocale(QLocale("C"))
                control.setValidator(val_obj)
                
        control.setEnabled(False)
        row_layout.addWidget(control, 2)
        
        form_layout.addRow(QLabel(attr), row_widget)
        
        key = f"{element}_{attr}"
        self.synth_widgets[key] = {
            "cb": cb,
            "control": control,
            "lbl_stock": lbl_stock,
            "stock_val": stock_val  # May be None for global entries
        }
        
        cb.blockSignals(True)
        control.blockSignals(True)
        
        if override_val is not None:
            cb.setChecked(True)
            control.setEnabled(True)
            if is_bool:
                control.setCurrentText(str(override_val))
            else:
                control.setText(str(override_val))
        else:
            cb.setChecked(False)
            control.setEnabled(False)
            if is_bool:
                control.setCurrentText(_stock_for_type)
            else:
                control.setText("")
                
        cb.blockSignals(False)
        control.blockSignals(False)
        
        cb.toggled.connect(control.setEnabled)
        cb.toggled.connect(lambda: self.validate_synth_highlights(key, filename))
        if is_bool:
            control.currentTextChanged.connect(lambda: self.validate_synth_highlights(key, filename))
        else:
            control.textChanged.connect(lambda: self.validate_synth_highlights(key, filename))
            
        self.validate_synth_highlights(key, filename, save=False)

    def validate_synth_highlights(self, key, filename, save=True):
        w_info = self.synth_widgets.get(key)
        if not w_info:
            return
            
        stock_val = w_info["stock_val"]
        cb = w_info["cb"]
        control = w_info["control"]
        lbl_stock = w_info["lbl_stock"]
        
        _stock_str = stock_val if stock_val is not None else ""
        is_bool = _stock_str.lower() in ["true", "false"]
        active_val = None
        if cb.isChecked():
            active_val = control.currentText() if is_bool else control.text()
        else:
            active_val = stock_val  # None for globals when unchecked
            
        is_diff = False
        if filename.startswith("[Global ") or stock_val is None:
            if cb.isChecked() and active_val is not None and active_val != "":
                is_diff = True
        else:
            if active_val != stock_val:
                try:
                    if float(active_val) != float(stock_val):
                        is_diff = True
                except (ValueError, TypeError):
                    is_diff = True
                    
        if is_diff:
            control.setStyleSheet("background-color: #3b3b1a; border: 1px solid #d4af37;")
            lbl_stock.setStyleSheet("color: #d4af37; font-weight: bold;")
        else:
            control.setStyleSheet("")
            # Preserve correct colour for global vs stock labels
            if stock_val is None:
                lbl_stock.setStyleSheet("color: #6888aa; font-style: italic;")
            else:
                lbl_stock.setStyleSheet("color: #a0a0aa; font-weight: normal;")
            
        if save:
            self.save_synth_from_ui(filename, key)

    def save_synth_from_ui(self, filename, key):
        if "synth_overrides" not in self.cfg.data:
            self.cfg.data["synth_overrides"] = {}
            
        if filename not in self.cfg.data["synth_overrides"]:
            self.cfg.data["synth_overrides"][filename] = {}
            
        element, attr = key.split("_", 1)
        w_info = self.synth_widgets[key]
        cb = w_info["cb"]
        control = w_info["control"]
        
        _stock_val = w_info["stock_val"]
        _stock_str = _stock_val if _stock_val is not None else ""
        is_bool = _stock_str.lower() in ["true", "false"]
        
        if cb.isChecked():
            val = control.currentText() if is_bool else control.text().replace(",", ".")
            if element not in self.cfg.data["synth_overrides"][filename]:
                self.cfg.data["synth_overrides"][filename][element] = {}
            self.cfg.data["synth_overrides"][filename][element][attr] = val
        else:
            if element in self.cfg.data["synth_overrides"][filename]:
                if attr in self.cfg.data["synth_overrides"][filename][element]:
                    del self.cfg.data["synth_overrides"][filename][element][attr]
                if not self.cfg.data["synth_overrides"][filename][element]:
                    del self.cfg.data["synth_overrides"][filename][element]
                    
        if not self.cfg.data["synth_overrides"][filename]:
            del self.cfg.data["synth_overrides"][filename]
            
        self.cfg.save_config()

    def action_clear_synth_overrides(self):
        current_item = self.list_synths.currentItem()
        if not current_item:
            return
        filename = current_item.text()
        
        reply = QMessageBox.question(
            self, 
            "Clear Overrides", 
            f"Are you sure you want to clear all overrides for {filename}?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.No:
            return
            
        if "synth_overrides" in self.cfg.data and filename in self.cfg.data["synth_overrides"]:
            del self.cfg.data["synth_overrides"][filename]
            self.cfg.save_config()
            
        self.build_synth_controls(filename)

    # ─────────────────────────────────────────────────────────────────────────
    # Profiles Tab
    # ─────────────────────────────────────────────────────────────────────────

    def init_profiles_tab(self):
        tab = QWidget()
        outer = QVBoxLayout(tab)
        outer.setContentsMargins(0, 0, 0, 0)
        outer.setSpacing(0)

        # ── Header banner ──────────────────────────────────────────────────
        banner = QWidget()
        banner.setStyleSheet(
            "background: qlineargradient(x1:0, y1:0, x2:1, y2:0, "
            "stop:0 #1a1a2e, stop:1 #16213e); border-bottom: 1px solid #2a2a45;"
        )
        banner_layout = QHBoxLayout(banner)
        banner_layout.setContentsMargins(20, 14, 20, 14)

        lbl_title = QLabel("Sound Profiles")
        lbl_title.setFont(QFont("Arial", 14, QFont.Weight.Bold))
        lbl_title.setStyleSheet("color: #c0c8ff; border: none;")
        banner_layout.addWidget(lbl_title)

        lbl_sub = QLabel("Save, load, and switch between complete sets of sound overrides")
        lbl_sub.setStyleSheet("color: #7080aa; border: none;")
        banner_layout.addWidget(lbl_sub)
        banner_layout.addStretch()

        self.lbl_active_profile_banner = QLabel("")
        self.lbl_active_profile_banner.setStyleSheet(
            "color: #80ff80; font-weight: bold; font-size: 10pt; border: none;"
        )
        banner_layout.addWidget(self.lbl_active_profile_banner)
        outer.addWidget(banner)

        # ── Main body ──────────────────────────────────────────────────────
        body = QWidget()
        body_layout = QHBoxLayout(body)
        body_layout.setContentsMargins(0, 0, 0, 0)
        body_layout.setSpacing(0)

        # ── Left pane: list + bottom buttons ──────────────────────────────
        left_pane = QWidget()
        left_pane.setMaximumWidth(300)
        left_pane.setStyleSheet("background-color: #15151e; border-right: 1px solid #2a2a3a;")
        left_layout = QVBoxLayout(left_pane)
        left_layout.setContentsMargins(12, 12, 12, 12)
        left_layout.setSpacing(8)

        lbl_list_title = QLabel("Saved Profiles")
        lbl_list_title.setStyleSheet("color: #8888aa; font-size: 9pt; font-weight: bold; border: none;")
        left_layout.addWidget(lbl_list_title)

        self.list_profiles = QListWidget()
        self.list_profiles.setStyleSheet(
            "QListWidget { background-color: #1a1a26; border: 1px solid #2a2a40; border-radius: 6px; }"
            "QListWidget::item { padding: 8px 10px; border-bottom: 1px solid #22223a; }"
            "QListWidget::item:selected { background-color: #2e3a6e; color: #c8d4ff; }"
            "QListWidget::item:hover { background-color: #22263a; }"
        )
        self.list_profiles.currentItemChanged.connect(self.profile_selection_changed)
        left_layout.addWidget(self.list_profiles, 1)

        btn_row = QHBoxLayout()
        btn_new = QPushButton("＋ New")
        btn_new.setObjectName("btn_profile_new")
        btn_new.setStyleSheet(
            "QPushButton { background-color: #1a3a2a; border: 1px solid #2a6a4a; color: #70ff90; "
            "border-radius: 4px; padding: 5px 10px; } "
            "QPushButton:hover { background-color: #225a3a; }"
        )
        btn_new.clicked.connect(self.action_new_profile)
        btn_row.addWidget(btn_new)

        btn_dup = QPushButton("⧉ Duplicate")
        btn_dup.clicked.connect(self.action_duplicate_profile)
        btn_row.addWidget(btn_dup)
        left_layout.addLayout(btn_row)

        body_layout.addWidget(left_pane)

        # ── Right pane: profile editor ─────────────────────────────────────
        right_pane = QScrollArea()
        right_pane.setWidgetResizable(True)
        right_pane.setFrameShape(QFrame.Shape.NoFrame if hasattr(QFrame, "Shape") else QFrame.NoFrame)
        right_pane.setStyleSheet("background-color: #1e1e2a;")

        right_content = QWidget()
        right_content.setStyleSheet("background-color: #1e1e2a;")
        right_v = QVBoxLayout(right_content)
        right_v.setContentsMargins(24, 20, 24, 20)
        right_v.setSpacing(16)

        # Welcome / empty state
        self.lbl_profile_welcome = QLabel("Select a profile from the list, or create a new one.")
        self.lbl_profile_welcome.setAlignment(get_align_center())
        self.lbl_profile_welcome.setStyleSheet(
            "color: #5060aa; font-size: 12pt; border: none;"
        )
        right_v.addWidget(self.lbl_profile_welcome)

        # Profile editor form (hidden until selection)
        self.widget_profile_editor = QWidget()
        self.widget_profile_editor.setVisible(False)
        editor_v = QVBoxLayout(self.widget_profile_editor)
        editor_v.setContentsMargins(0, 0, 0, 0)
        editor_v.setSpacing(14)

        # Name row
        name_grp = QGroupBox("Profile Name")
        name_layout = QHBoxLayout(name_grp)
        self.edit_profile_name = QLineEdit()
        self.edit_profile_name.setPlaceholderText("Profile name…")
        name_layout.addWidget(self.edit_profile_name, 1)
        btn_rename = QPushButton("✏ Rename")
        btn_rename.clicked.connect(self.action_rename_profile)
        name_layout.addWidget(btn_rename)
        editor_v.addWidget(name_grp)

        # Description
        desc_grp = QGroupBox("Description (optional)")
        desc_layout = QVBoxLayout(desc_grp)
        self.edit_profile_desc = QTextEdit()
        self.edit_profile_desc.setMaximumHeight(80)
        self.edit_profile_desc.setPlaceholderText("Add a description for this profile…")
        self.edit_profile_desc.setStyleSheet(
            "background-color: #2b2b38; border: 1px solid #3a3a50; color: #d0d0e0; border-radius: 4px; padding: 4px;"
        )
        desc_layout.addWidget(self.edit_profile_desc)
        btn_save_desc = QPushButton("Save Description")
        btn_save_desc.clicked.connect(self.action_save_profile_description)
        desc_layout.addWidget(btn_save_desc, 0, get_align_left() if hasattr(get_align_left(), "__int__") else get_align_left())
        editor_v.addWidget(desc_grp)

        # Stats
        self.lbl_profile_stats = QLabel("")
        self.lbl_profile_stats.setStyleSheet("color: #6878a8; font-size: 9pt; border: none;")
        editor_v.addWidget(self.lbl_profile_stats)

        # Separator
        sep = QFrame()
        sep.setFrameShape(QFrame.Shape.HLine if hasattr(QFrame, "Shape") else QFrame.HLine)
        sep.setStyleSheet("color: #2a2a40;")
        editor_v.addWidget(sep)

        # Action buttons
        actions_grp = QGroupBox("Profile Actions")
        actions_layout = QVBoxLayout(actions_grp)
        actions_layout.setSpacing(8)

        btn_save_here = QPushButton("💾  Save Current Settings to This Profile")
        btn_save_here.setStyleSheet(
            "QPushButton { background-color: #1e2e4a; border: 1px solid #2a4a7a; color: #80b0ff; "
            "border-radius: 4px; padding: 8px 14px; text-align: left; } "
            "QPushButton:hover { background-color: #253666; }"
        )
        btn_save_here.clicked.connect(self.action_save_to_profile)
        actions_layout.addWidget(btn_save_here)

        btn_load = QPushButton("📂  Load Profile into Editor")
        btn_load.setStyleSheet(
            "QPushButton { background-color: #2a2e1e; border: 1px solid #5a6a2a; color: #b0d860; "
            "border-radius: 4px; padding: 8px 14px; text-align: left; } "
            "QPushButton:hover { background-color: #3a4028; }"
        )
        btn_load.clicked.connect(self.action_load_profile)
        actions_layout.addWidget(btn_load)

        btn_apply_profile = QPushButton("▶  Apply Profile to Game")
        btn_apply_profile.setStyleSheet(
            "QPushButton { background-color: #1a3a1a; border: 1px solid #2a7a2a; color: #60e060; "
            "font-weight: bold; border-radius: 4px; padding: 8px 14px; text-align: left; } "
            "QPushButton:hover { background-color: #225522; }"
        )
        btn_apply_profile.clicked.connect(self.action_apply_profile)
        actions_layout.addWidget(btn_apply_profile)

        btn_delete = QPushButton("🗑  Delete Profile")
        btn_delete.setObjectName("btn_restore")
        btn_delete.setStyleSheet(
            "QPushButton { background-color: #3a1a1a; border: 1px solid #7a2a2a; color: #e06060; "
            "border-radius: 4px; padding: 8px 14px; text-align: left; } "
            "QPushButton:hover { background-color: #552222; }"
        )
        btn_delete.clicked.connect(self.action_delete_profile)
        actions_layout.addWidget(btn_delete)
        editor_v.addWidget(actions_grp)

        editor_v.addStretch()
        right_v.addWidget(self.widget_profile_editor, 1)
        right_v.addStretch()
        right_pane.setWidget(right_content)
        body_layout.addWidget(right_pane, 1)
        outer.addWidget(body, 1)

        self.tab_widget.addTab(tab, "Profiles")
        # Populate on start
        self.populate_profiles_list()

    def populate_profiles_list(self):
        self.list_profiles.clear()
        profiles = self.cfg.data.get("profiles", {})
        active = self.cfg.data.get("active_profile", "")
        for name in sorted(profiles.keys()):
            item_text = f"★ {name}" if name == active else f"  {name}"
            self.list_profiles.addItem(item_text)
        self._update_active_profile_banner()

    def _profile_name_from_item(self, item):
        if not item:
            return None
        text = item.text().strip()
        # Remove the ★ marker if present
        if text.startswith("★"):
            text = text[1:].strip()
        return text

    def _update_active_profile_banner(self):
        active = self.cfg.data.get("active_profile", "")
        if active:
            self.lbl_active_profile_banner.setText(f"Active: {active}")
        else:
            self.lbl_active_profile_banner.setText("")

    def profile_selection_changed(self, current_item):
        if not current_item:
            self.lbl_profile_welcome.setVisible(True)
            self.widget_profile_editor.setVisible(False)
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if name not in profiles:
            return
        profile = profiles[name]
        self.lbl_profile_welcome.setVisible(False)
        self.widget_profile_editor.setVisible(True)

        self.edit_profile_name.setText(name)
        self.edit_profile_desc.setPlainText(profile.get("description", ""))

        # Stats
        n_car = sum(1 for v in profile.get("car_overrides", {}).values() if v)
        n_global = sum(1 for v in profile.get("global_overrides", {}).values() if v is not None)
        n_synth = len(profile.get("synth_overrides", {}))
        n_misc = sum(1 for v in profile.get("misc_overrides", {}).values() if v is not None)
        created = profile.get("created", "—")
        active = self.cfg.data.get("active_profile", "")
        active_tag = "  ✓ ACTIVE" if name == active else ""
        self.lbl_profile_stats.setText(
            f"Created: {created}{active_tag}\n"
            f"Car overrides: {n_car} cars  ·  "
            f"Global overrides: {n_global}  ·  "
            f"Class overrides: {n_synth} files  ·  "
            f"Misc overrides: {n_misc}"
        )

    def get_settings_snapshot(self):
        """Returns a deep copy of all current override settings."""
        return {
            "car_overrides": copy.deepcopy(self.cfg.data.get("car_overrides", {})),
            "global_overrides": copy.deepcopy(self.cfg.data.get("global_overrides", {})),
            "synth_overrides": copy.deepcopy(self.cfg.data.get("synth_overrides", {})),
            "misc_overrides": copy.deepcopy(self.cfg.data.get("misc_overrides", {})),
        }

    def apply_snapshot_to_ui(self, snapshot):
        """Loads a profile snapshot into cfg.data then refreshes all tab widgets."""
        self.cfg.data["car_overrides"] = copy.deepcopy(snapshot.get("car_overrides", {}))
        self.cfg.data["global_overrides"] = copy.deepcopy(snapshot.get("global_overrides", {}))
        self.cfg.data["synth_overrides"] = copy.deepcopy(snapshot.get("synth_overrides", {}))
        self.cfg.data["misc_overrides"] = copy.deepcopy(snapshot.get("misc_overrides", {}))
        self.cfg.save_config()

        # Reload global_widgets
        global_overrides = self.cfg.data["global_overrides"]
        for key, w_info in self.global_widgets.items():
            w_info["cb"].blockSignals(True)
            w_info["control"].blockSignals(True)
            val = global_overrides.get(key)
            if val is not None:
                w_info["cb"].setChecked(True)
                w_info["control"].setEnabled(True)
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText(str(val))
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText(str(val))
            else:
                w_info["cb"].setChecked(False)
                w_info["control"].setEnabled(False)
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText("")
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText("")
            w_info["cb"].blockSignals(False)
            w_info["control"].blockSignals(False)

        # Reload car_widgets if a car is selected
        current_car = self.list_cars.currentItem()
        if current_car:
            self.car_selection_changed(current_car)

        # Reload synth_widgets if a synth is selected
        current_synth = self.list_synths.currentItem()
        if current_synth:
            self.synth_selection_changed(current_synth)

        # Reload misc widgets
        misc_overrides = self.cfg.data.get("misc_overrides", {})
        for (cb_ctrl, edit_ctrl, key, default_val) in [
            (self.cb_misc_blur, self.edit_misc_blur, "blur_rim_max_rpm", "350000.0"),
            (self.cb_misc_bfmin, self.edit_misc_bfmin, "backfire_min_rand_time", "100.0"),
            (self.cb_misc_bfmax, self.edit_misc_bfmax, "backfire_max_rand_time", "350.0"),
        ]:
            cb_ctrl.blockSignals(True)
            edit_ctrl.blockSignals(True)
            if key in misc_overrides:
                cb_ctrl.setChecked(True)
                edit_ctrl.setEnabled(True)
                edit_ctrl.setText(str(misc_overrides[key]))
            else:
                cb_ctrl.setChecked(False)
                edit_ctrl.setEnabled(False)
                edit_ctrl.setText(default_val)
            cb_ctrl.blockSignals(False)
            edit_ctrl.blockSignals(False)

        self.validate_misc_highlights()
        self.status_bar.showMessage("Profile loaded into editor")

    # ── Profile CRUD actions ───────────────────────────────────────────────

    def action_new_profile(self):
        name, ok = QInputDialog.getText(self, "New Profile", "Enter a name for the new profile:")
        if not ok or not name.strip():
            return
        name = name.strip()
        profiles = self.cfg.data.setdefault("profiles", {})
        if name in profiles:
            QMessageBox.warning(self, "Name Taken", f"A profile named '{name}' already exists.")
            return
        snapshot = self.get_settings_snapshot()
        snapshot["description"] = ""
        snapshot["created"] = datetime.datetime.now().isoformat(timespec="seconds")
        self.cfg.save_profile(name, snapshot)
        self.populate_profiles_list()
        # Select the new item
        for i in range(self.list_profiles.count()):
            n = self._profile_name_from_item(self.list_profiles.item(i))
            if n == name:
                self.list_profiles.setCurrentRow(i)
                break
        self.status_bar.showMessage(f"Profile '{name}' created from current settings")

    def action_duplicate_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            QMessageBox.information(self, "No Selection", "Select a profile to duplicate.")
            return
        source_name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if source_name not in profiles:
            return
        default_new = f"{source_name} (copy)"
        name, ok = QInputDialog.getText(self, "Duplicate Profile", "Name for the duplicate:", text=default_new)
        if not ok or not name.strip():
            return
        name = name.strip()
        if name in profiles:
            QMessageBox.warning(self, "Name Taken", f"A profile named '{name}' already exists.")
            return
        new_profile = copy.deepcopy(profiles[source_name])
        new_profile["created"] = datetime.datetime.now().isoformat(timespec="seconds")
        self.cfg.save_profile(name, new_profile)
        self.populate_profiles_list()
        for i in range(self.list_profiles.count()):
            if self._profile_name_from_item(self.list_profiles.item(i)) == name:
                self.list_profiles.setCurrentRow(i)
                break
        self.status_bar.showMessage(f"Profile '{name}' duplicated from '{source_name}'")

    def action_rename_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        old_name = self._profile_name_from_item(current_item)
        new_name = self.edit_profile_name.text().strip()
        if not new_name or new_name == old_name:
            return
        profiles = self.cfg.data.get("profiles", {})
        if new_name in profiles:
            QMessageBox.warning(self, "Name Taken", f"A profile named '{new_name}' already exists.")
            return
        
        # Save under new name, delete old file
        profile_data = profiles[old_name]
        self.cfg.save_profile(new_name, profile_data)
        self.cfg.delete_profile(old_name)
        
        if self.cfg.data.get("active_profile") == old_name:
            self.cfg.data["active_profile"] = new_name
            self.cfg.save_config()
            
        self.populate_profiles_list()
        for i in range(self.list_profiles.count()):
            if self._profile_name_from_item(self.list_profiles.item(i)) == new_name:
                self.list_profiles.setCurrentRow(i)
                break
        self.status_bar.showMessage(f"Profile renamed to '{new_name}'")

    def action_save_profile_description(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if name not in profiles:
            return
        profiles[name]["description"] = self.edit_profile_desc.toPlainText()
        self.cfg.save_profile(name, profiles[name])
        self.status_bar.showMessage("Description saved")

    def action_delete_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if len(profiles) <= 1:
            QMessageBox.warning(self, "Cannot Delete", "You must have at least one profile.")
            return
        reply = QMessageBox.question(
            self, "Delete Profile",
            f"Are you sure you want to delete '{name}'?\nThis cannot be undone.",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.No:
            return
        self.cfg.delete_profile(name)
        if self.cfg.data.get("active_profile") == name:
            # Switch to first remaining profile
            first = next(iter(self.cfg.data.get("profiles", {})), "")
            self.cfg.data["active_profile"] = first
            self.cfg.save_config()
            
        self.populate_profiles_list()
        if self.list_profiles.count() > 0:
            self.list_profiles.setCurrentRow(0)
        self.lbl_profile_welcome.setVisible(True)
        self.widget_profile_editor.setVisible(False)
        self.status_bar.showMessage(f"Profile '{name}' deleted")

    def action_save_to_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if name not in profiles:
            return
        self.save_global_from_ui()
        self.save_misc_from_ui()
        snapshot = self.get_settings_snapshot()
        snapshot["description"] = profiles[name].get("description", "")
        snapshot["created"] = profiles[name].get("created", datetime.datetime.now().isoformat(timespec="seconds"))
        
        self.cfg.save_profile(name, snapshot)
        self.profile_selection_changed(current_item)  # refresh stats
        self.status_bar.showMessage(f"Current settings saved to profile '{name}'")
        QMessageBox.information(self, "Saved", f"Settings saved to profile '{name}'.")

    def action_load_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if name not in profiles:
            return
        reply = QMessageBox.question(
            self, "Load Profile",
            f"Load '{name}'? This will replace all current settings in the editor.",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.No:
            return
        self.cfg.data["active_profile"] = name
        self.apply_snapshot_to_ui(profiles[name])
        self.populate_profiles_list()
        # Re-select the same item after repopulating
        for i in range(self.list_profiles.count()):
            if self._profile_name_from_item(self.list_profiles.item(i)) == name:
                self.list_profiles.setCurrentRow(i)
                break

    def action_apply_profile(self):
        current_item = self.list_profiles.currentItem()
        if not current_item:
            return
        name = self._profile_name_from_item(current_item)
        profiles = self.cfg.data.get("profiles", {})
        if name not in profiles:
            return
        has_backup, _ = self.engine.check_backup_status()
        if not has_backup:
            QMessageBox.warning(self, "No Backup", "Please create a backup on the Setup tab first.")
            return
        # Load the profile settings, then apply
        self.cfg.data["active_profile"] = name
        self.apply_snapshot_to_ui(profiles[name])
        self.populate_profiles_list()

        self.status_bar.showMessage(f"Applying profile '{name}' to game files…")
        self.progress_bar.setVisible(True)
        self.progress_bar.setValue(0)
        self.setEnabled(False)
        try:
            self.engine.apply_overrides(self.progress_bar.setValue)
            QMessageBox.information(
                self, "Success",
                f"Profile '{name}' applied to game files successfully!"
            )
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to apply profile:\n{e}")
        finally:
            self.setEnabled(True)
            self.progress_bar.setVisible(False)
            self.status_bar.showMessage("Ready")

    def init_misc_tab(self):

        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(20, 20, 20, 20)

        group_misc = QGroupBox("Global Vehicle Attributes Override")
        layout_misc = QGridLayout(group_misc)
        layout_misc.setContentsMargins(15, 15, 15, 15)
        layout_misc.setHorizontalSpacing(15)
        layout_misc.setVerticalSpacing(15)

        layout_misc.addWidget(QLabel("Attribute"), 0, 0)
        layout_misc.addWidget(QLabel("Original Stock Value"), 0, 1)
        layout_misc.addWidget(QLabel("Override?"), 0, 2)
        layout_misc.addWidget(QLabel("New Override Value"), 0, 3)

        # 1. BlurRim MaxRPM
        layout_misc.addWidget(QLabel("BlurRim Rotation Limit (MaxRPM)"), 1, 0)
        self.lbl_misc_blur_stock = QLabel("Stock: 350.0")
        self.lbl_misc_blur_stock.setStyleSheet("color: #a0a0aa;")
        layout_misc.addWidget(self.lbl_misc_blur_stock, 1, 1)
        self.cb_misc_blur = QCheckBox()
        layout_misc.addWidget(self.cb_misc_blur, 1, 2)
        self.edit_misc_blur = QLineEdit()
        v1 = QDoubleValidator(0, 1000000, 2, self.edit_misc_blur)
        v1.setLocale(QLocale("C"))
        self.edit_misc_blur.setValidator(v1)
        self.edit_misc_blur.setEnabled(False)
        layout_misc.addWidget(self.edit_misc_blur, 1, 3)

        self.cb_misc_blur.toggled.connect(self.edit_misc_blur.setEnabled)
        self.cb_misc_blur.toggled.connect(self.validate_misc_highlights)
        self.edit_misc_blur.textChanged.connect(self.validate_misc_highlights)

        # 2. BackfireMinRandTime
        layout_misc.addWidget(QLabel("Backfire Minimum Delay (ms)"), 2, 0)
        self.lbl_misc_bfmin_stock = QLabel("Stock: 100.0")
        self.lbl_misc_bfmin_stock.setStyleSheet("color: #a0a0aa;")
        layout_misc.addWidget(self.lbl_misc_bfmin_stock, 2, 1)
        self.cb_misc_bfmin = QCheckBox()
        layout_misc.addWidget(self.cb_misc_bfmin, 2, 2)
        self.edit_misc_bfmin = QLineEdit()
        v2 = QDoubleValidator(0, 5000, 2, self.edit_misc_bfmin)
        v2.setLocale(QLocale("C"))
        self.edit_misc_bfmin.setValidator(v2)
        self.edit_misc_bfmin.setEnabled(False)
        layout_misc.addWidget(self.edit_misc_bfmin, 2, 3)

        self.cb_misc_bfmin.toggled.connect(self.edit_misc_bfmin.setEnabled)
        self.cb_misc_bfmin.toggled.connect(self.validate_misc_highlights)
        self.edit_misc_bfmin.textChanged.connect(self.validate_misc_highlights)

        # 3. BackfireMaxRandTime
        layout_misc.addWidget(QLabel("Backfire Maximum Delay (ms)"), 3, 0)
        self.lbl_misc_bfmax_stock = QLabel("Stock: 350.0")
        self.lbl_misc_bfmax_stock.setStyleSheet("color: #a0a0aa;")
        layout_misc.addWidget(self.lbl_misc_bfmax_stock, 3, 1)
        self.cb_misc_bfmax = QCheckBox()
        layout_misc.addWidget(self.cb_misc_bfmax, 3, 2)
        self.edit_misc_bfmax = QLineEdit()
        v3 = QDoubleValidator(0, 5000, 2, self.edit_misc_bfmax)
        v3.setLocale(QLocale("C"))
        self.edit_misc_bfmax.setValidator(v3)
        self.edit_misc_bfmax.setEnabled(False)
        layout_misc.addWidget(self.edit_misc_bfmax, 3, 3)

        self.cb_misc_bfmax.toggled.connect(self.edit_misc_bfmax.setEnabled)
        self.cb_misc_bfmax.toggled.connect(self.validate_misc_highlights)
        self.edit_misc_bfmax.textChanged.connect(self.validate_misc_highlights)

        layout.addWidget(group_misc)
        layout.addStretch()

        self.tab_widget.addTab(tab, "Miscellaneous")

    def init_reverb_tab(self):
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(20, 20, 20, 20)
        
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        
        group = QGroupBox("Global Reverb Multipliers")
        g_layout = QGridLayout(group)
        g_layout.setContentsMargins(15, 15, 15, 15)
        g_layout.setHorizontalSpacing(15)
        g_layout.setVerticalSpacing(10)
        
        g_layout.addWidget(QLabel("Attribute"), 0, 0)
        g_layout.addWidget(QLabel("Override Multiplier?"), 0, 1)
        g_layout.addWidget(QLabel("Multiplier (1.0 = Stock)"), 0, 2)
        g_layout.addWidget(QLabel("Override Offset?"), 0, 3)
        g_layout.addWidget(QLabel("Offset (+/-)"), 0, 4)
        
        self.reverb_widgets = {}
        
        reverb_fields = [
            "TrackReflectionDist", "NearDist", "FarDist", "NearDry", "NearWet", 
            "FarDry", "FarWet", "DryLevelmB", "RoomLevelmB", "RoomHFLevelmB", 
            "RoomRolloffFactor", "DecayTimeSec", "DecayHFRatio", "ReflectionsLevelmB", 
            "ReflectionsDelaySec", "ReverbLevelmB", "ReverbDelaySec", "DiffusionPercent", 
            "DensityPercent", "HFReferenceHz"
        ]
        
        reverb_overrides = self.cfg.data.get("reverb_overrides", {})
        
        for idx, field in enumerate(reverb_fields):
            row = idx + 1
            g_layout.addWidget(QLabel(field), row, 0)
            
            cb = QCheckBox()
            g_layout.addWidget(cb, row, 1)
            
            spin = QDoubleSpinBox()
            spin.setRange(-100.0, 100.0)
            spin.setSingleStep(0.1)
            spin.setDecimals(3)
            spin.setValue(1.0)
            spin.setEnabled(False)
            g_layout.addWidget(spin, row, 2)
            
            cb_off = QCheckBox()
            g_layout.addWidget(cb_off, row, 3)
            
            spin_off = QDoubleSpinBox()
            spin_off.setRange(-20000.0, 20000.0)
            spin_off.setSingleStep(100.0)
            spin_off.setDecimals(3)
            spin_off.setValue(0.0)
            spin_off.setEnabled(False)
            g_layout.addWidget(spin_off, row, 4)
            
            if field in reverb_overrides:
                cb.setChecked(True)
                spin.setEnabled(True)
                spin.setValue(float(reverb_overrides[field]))
                
            offset_key = field + "_offset"
            if offset_key in reverb_overrides:
                cb_off.setChecked(True)
                spin_off.setEnabled(True)
                spin_off.setValue(float(reverb_overrides[offset_key]))
                
            cb.toggled.connect(spin.setEnabled)
            cb.toggled.connect(self.save_reverb_config)
            spin.valueChanged.connect(self.save_reverb_config)
            
            cb_off.toggled.connect(spin_off.setEnabled)
            cb_off.toggled.connect(self.save_reverb_config)
            spin_off.valueChanged.connect(self.save_reverb_config)
            
            self.reverb_widgets[field] = (cb, spin, cb_off, spin_off)
            
        scroll_layout.addWidget(group)
        scroll_layout.addStretch()
        scroll.setWidget(scroll_content)
        layout.addWidget(scroll)
        self.tab_widget.addTab(tab, "Reverb")

    def save_reverb_config(self):
        if "reverb_overrides" not in self.cfg.data:
            self.cfg.data["reverb_overrides"] = {}
        
        overrides = {}
        for field, widgets in self.reverb_widgets.items():
            cb, spin, cb_off, spin_off = widgets
            if cb.isChecked():
                overrides[field] = spin.value()
            if cb_off.isChecked():
                overrides[field + "_offset"] = spin_off.value()
                
        self.cfg.data["reverb_overrides"] = overrides
        self.cfg.save_config()

    # --- Setup Tab Operations ---
    def browse_game_path(self):
        dir_path = QFileDialog.getExistingDirectory(self, "Select Forza Horizon 6 Game Directory", self.edit_game_path.text())
        if dir_path:
            self.edit_game_path.setText(dir_path)

    def game_path_changed(self, text):
        self.cfg.data["game_path"] = text
        self.cfg.save_config()
        self.update_backup_status()
        self.load_options_and_scan()

    def update_backup_status(self):
        has_backup, status_msg = self.engine.check_backup_status()
        self.lbl_backup_status.setText(f"Backup Status: {status_msg}")
        self.btn_restore.setEnabled(has_backup)

        # Verify if path doesn't end in ForzaHorizon6 and warn (only once when path changes)
        path = self.cfg.data["game_path"]
        if path:
            norm_path = os.path.normpath(path)
            folder_name = os.path.basename(norm_path)
            if folder_name != "ForzaHorizon6":
                self.lbl_backup_status.setText(f"Backup Status: {status_msg} (WARNING: Directory name does not end with 'ForzaHorizon6')")

    def action_create_backup(self):
        game_path = self.cfg.data["game_path"]
        norm_path = os.path.normpath(game_path)
        folder_name = os.path.basename(norm_path)

        if folder_name != "ForzaHorizon6":
            reply = QMessageBox.warning(
                self,
                "Warning",
                f"The selected folder name '{folder_name}' is not 'ForzaHorizon6'.\nDo you want to proceed anyway?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
            )
            if reply == QMessageBox.StandardButton.No:
                return

        self.status_bar.showMessage("Creating XML backup...")
        self.progress_bar.setVisible(True)
        self.progress_bar.setValue(0)
        self.setEnabled(False)

        # In standard PySide GUI, running heavy tasks block. Since XML copies are small, we can run synchronously safely.
        try:
            self.engine.create_backup(self.progress_bar.setValue)
            QMessageBox.information(self, "Success", "Backup of XML files created successfully!")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to create backup:\n{e}")
        finally:
            self.setEnabled(True)
            self.progress_bar.setVisible(False)
            self.update_backup_status()
            self.load_options_and_scan()

    def action_restore(self):
        reply = QMessageBox.question(
            self,
            "Restore Game Defaults",
            "Are you sure you want to restore all sound configuration files back to stock?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.No:
            return

        self.status_bar.showMessage("Restoring files...")
        self.setEnabled(False)
        try:
            self.engine.restore_backup()
            QMessageBox.information(self, "Success", "Restored stock game files successfully!")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to restore backup:\n{e}")
        finally:
            self.setEnabled(True)
            self.status_bar.showMessage("Ready")

    # --- Scan and Data Loading ---
    def load_options_and_scan(self):
        """Loads unique choice lists from game config and populates combobox lists."""
        has_backup, _ = self.engine.check_backup_status()
        if not has_backup:
            # Disable configuration tabs
            self.tab_widget.setTabEnabled(1, False)
            self.tab_widget.setTabEnabled(2, False)
            self.tab_widget.setTabEnabled(3, False)
            self.tab_widget.setTabEnabled(4, False)
            return

        self.tab_widget.setTabEnabled(1, True)
        self.tab_widget.setTabEnabled(2, True)
        self.tab_widget.setTabEnabled(3, True)
        self.tab_widget.setTabEnabled(4, True)


        self.status_bar.showMessage("Scanning game XML files...")
        self.engine.scan_cars()
        self.engine.scan_synths()

        # Populate global widgets' comboboxes with dynamic choices scraped
        for key, w_info in self.global_widgets.items():
            ctrl = w_info["control"]
            if isinstance(ctrl, QComboBox):
                # Clean choices
                ctrl.clear()
                ctrl.addItem("")

                # Check channel type
                ch = None
                for chan in ["Engine", "Exhaust", "Intake"]:
                    if key.startswith(chan):
                        ch = chan
                        break

                if ch:
                    # Dynamically scraped
                    sorted_options = sorted(list(self.engine.scraped_options[ch]))
                    ctrl.addItems(sorted_options)
                else:
                    # Hardcoded lists
                    if "Transmission" in key:
                        ctrl.addItems(TRANSMISSION_PROFILES)
                    elif "Turbo" in key:
                        ctrl.addItems(TURBO_PROFILES)
                    elif "SuperCSC" in key:
                        ctrl.addItems(SUPER_CSC_PROFILES)
                    elif "SuperDSC" in key:
                        ctrl.addItems(SUPER_DSC_PROFILES)
                    elif "Burbles" in key:
                        ctrl.addItems(BURBLES_OPTIONS)
                    elif "Backfire" in key or "AntiLag" in key:
                        ctrl.addItems(BACKFIRE_ANTILAG_OPTIONS)
                    elif "BOV" in key:
                        ctrl.addItems(BOV_OPTIONS)
                    elif "GearCrack" in key:
                        ctrl.addItems(GEARCRACK_OPTIONS)
                    elif "Limiter" in key:
                        ctrl.addItems(LIMITER_OPTIONS)
                    elif "Throttle" in key:
                        ctrl.addItems(THROTTLE_OPTIONS)
                    elif "EngineBank" in key:
                        ctrl.addItems(ENGINE_BANK_OPTIONS)

        # Update per-car widgets lists as well
        for key, w_info in self.car_widgets.items():
            ctrl = w_info["control"]
            if isinstance(ctrl, QComboBox):
                ctrl.clear()
                ctrl.addItem("")
                ch = None
                for chan in ["Engine", "Exhaust", "Intake"]:
                    if key.startswith(chan):
                        ch = chan
                        break
                if ch:
                    sorted_options = sorted(list(self.engine.scraped_options[ch]))
                    ctrl.addItems(sorted_options)
                else:
                    if "Transmission" in key:
                        ctrl.addItems(TRANSMISSION_PROFILES)
                    elif "Turbo" in key:
                        ctrl.addItems(TURBO_PROFILES)
                    elif "SuperCSC" in key:
                        ctrl.addItems(SUPER_CSC_PROFILES)
                    elif "SuperDSC" in key:
                        ctrl.addItems(SUPER_DSC_PROFILES)
                    elif "Burbles" in key:
                        ctrl.addItems(BURBLES_OPTIONS)
                    elif "Backfire" in key or "AntiLag" in key:
                        ctrl.addItems(BACKFIRE_ANTILAG_OPTIONS)
                    elif "BOV" in key:
                        ctrl.addItems(BOV_OPTIONS)
                    elif "GearCrack" in key:
                        ctrl.addItems(GEARCRACK_OPTIONS)
                    elif "Limiter" in key:
                        ctrl.addItems(LIMITER_OPTIONS)
                    elif "Throttle" in key:
                        ctrl.addItems(THROTTLE_OPTIONS)
                    elif "EngineBank" in key:
                        ctrl.addItems(ENGINE_BANK_OPTIONS)

        # Populate Setup configurations in global
        global_overrides = self.cfg.data.get("global_overrides", {})
        for key, w_info in self.global_widgets.items():
            val = global_overrides.get(key)
            if val is not None:
                w_info["cb"].setChecked(True)
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText(str(val))
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText(str(val))

        # Populate Car and Synthesizer lists
        self.populate_car_list()
        self.populate_synth_list()

        # Populate Misc values
        misc_stock = self.engine.read_stock_misc_values()
        self.lbl_misc_blur_stock.setText(f"Stock: {misc_stock['blur_rim_max_rpm']}")
        self.lbl_misc_bfmin_stock.setText(f"Stock: {misc_stock['backfire_min_rand_time']}")
        self.lbl_misc_bfmax_stock.setText(f"Stock: {misc_stock['backfire_max_rand_time']}")

        misc_overrides = self.cfg.data.get("misc_overrides", {})
        if "blur_rim_max_rpm" in misc_overrides:
            self.cb_misc_blur.setChecked(True)
            self.edit_misc_blur.setText(str(misc_overrides["blur_rim_max_rpm"]))
        else:
            self.edit_misc_blur.setText("350000.0")

        if "backfire_min_rand_time" in misc_overrides:
            self.cb_misc_bfmin.setChecked(True)
            self.edit_misc_bfmin.setText(str(misc_overrides["backfire_min_rand_time"]))
        else:
            self.edit_misc_bfmin.setText("100.0")

        if "backfire_max_rand_time" in misc_overrides:
            self.cb_misc_bfmax.setChecked(True)
            self.edit_misc_bfmax.setText(str(misc_overrides["backfire_max_rand_time"]))
        else:
            self.edit_misc_bfmax.setText("350.0")

        self.status_bar.showMessage("Ready")

    # --- Global Overrides Operations ---
    def save_global_from_ui(self):
        """Reads UI controls and updates config dict for global overrides."""
        global_overrides = {}
        for key, w_info in self.global_widgets.items():
            if w_info["cb"].isChecked():
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    global_overrides[key] = ctrl.currentText()
                elif isinstance(ctrl, QLineEdit):
                    global_overrides[key] = ctrl.text().replace(",", ".")
            else:
                global_overrides[key] = None
        self.cfg.data["global_overrides"] = global_overrides
        self.cfg.save_config()

    # --- Car Tab Operations ---
    def populate_car_list(self):
        self.list_cars.clear()
        self.list_cars.addItem("[Global Overrides]")
        filter_text = self.search_cars.text().lower()
        for car in self.engine.car_list:
            display_name = car["id"].replace("_", " ")
            if filter_text in car["id"].lower() or filter_text in display_name.lower():
                self.list_cars.addItem(car["id"])

    def filter_car_list(self):
        self.populate_car_list()

    def car_selection_changed(self, current_item):
        if not current_item:
            self.lbl_per_car_welcome.setVisible(True)
            self.scroll_car_config.setVisible(False)
            self.scroll_global_config.setVisible(False)
            return

        car_id = current_item.text()

        if car_id == "[Global Overrides]":
            self.lbl_per_car_welcome.setVisible(False)
            self.scroll_car_config.setVisible(False)
            self.scroll_global_config.setVisible(True)
            return

        self.lbl_per_car_welcome.setVisible(False)
        self.scroll_global_config.setVisible(False)
        self.scroll_car_config.setVisible(True)
        self.lbl_selected_car_id.setText(car_id.replace("_", " "))

        # Load values
        car_info = next((c for c in self.engine.car_list if c["id"] == car_id), None)
        if not car_info:
            return

        # Temporarily disconnect signals to prevent cycle writes during loading
        for key, w_info in self.car_widgets.items():
            w_info["cb"].blockSignals(True)
            w_info["control"].blockSignals(True)

        stock_values = car_info["stock_values"]
        car_overrides = self.cfg.data["car_overrides"].get(car_id, {})

        for key, w_info in self.car_widgets.items():
            # 1. Update stock label
            stock_val = stock_values.get(key, "None")
            w_info["lbl_stock"].setText(f"Stock: {stock_val}")

            # 2. Check if car override exists
            if key in car_overrides and car_overrides[key] is not None:
                w_info["cb"].setChecked(True)
                w_info["control"].setEnabled(True)
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText(str(car_overrides[key]))
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText(str(car_overrides[key]))
            else:
                w_info["cb"].setChecked(False)
                w_info["control"].setEnabled(False)
                # Set input value to stock to look clean
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText("")
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText("")

        # Re-enable signals
        for key, w_info in self.car_widgets.items():
            w_info["cb"].blockSignals(False)
            w_info["control"].blockSignals(False)

        # Highlight differences
        for key in self.car_widgets:
            self.validate_car_highlights(key)

    def validate_car_highlights(self, key):
        """Checks if final active value for car differs from stock, highlighting if it does."""
        current_car = self.list_cars.currentItem()
        if not current_car:
            return
        car_id = current_car.text()
        if car_id == "[Global Overrides]":
            return

        car_info = next((c for c in self.engine.car_list if c["id"] == car_id), None)
        if not car_info:
            return

        stock_val = car_info["stock_values"].get(key)

        # Determine resulting active value
        w_info = self.car_widgets[key]
        active_val = None
        if w_info["cb"].isChecked():
            # Local override
            ctrl = w_info["control"]
            active_val = ctrl.text() if isinstance(ctrl, QLineEdit) else ctrl.currentText()
        else:
            # Check global override
            global_w = self.global_widgets.get(key)
            if global_w and global_w["cb"].isChecked():
                global_ctrl = global_w["control"]
                active_val = global_ctrl.text() if isinstance(global_ctrl, QLineEdit) else global_ctrl.currentText()
            else:
                active_val = stock_val

        # Compare values (convert floats if necessary)
        is_diff = False
        if active_val is not None and stock_val is not None:
            if active_val != stock_val:
                try:
                    if float(active_val) != float(stock_val):
                        is_diff = True
                except ValueError:
                    is_diff = True
        elif active_val != stock_val:
            is_diff = True

        # Apply CSS styling
        control = w_info["control"]
        lbl_stock = w_info["lbl_stock"]
        lbl_name = w_info["lbl_name"]

        if is_diff:
            highlight_qss = "background-color: #3b3b1a; border: 1px solid #d4af37;"
            control.setStyleSheet(highlight_qss)
            lbl_stock.setStyleSheet("color: #d4af37; font-weight: bold;")
            if lbl_name:
                lbl_name.setStyleSheet("color: #d4af37; font-weight: bold;")
        else:
            control.setStyleSheet("")
            lbl_stock.setStyleSheet("color: #a0a0aa; font-weight: normal;")
            if lbl_name:
                lbl_name.setStyleSheet("color: #e0e0e6; font-weight: normal;")

        # Enable/disable link button — always navigate to the best available target
        btn_link = w_info.get("btn_link")
        if btn_link:
            nav_val = None
            nav_source = ""
            if w_info["cb"].isChecked():
                ctrl = w_info["control"]
                nav_val = ctrl.text() if isinstance(ctrl, QLineEdit) else ctrl.currentText()
                if nav_val and nav_val.strip():
                    nav_source = "car override"
            if not nav_val or not nav_val.strip():
                global_w = self.global_widgets.get(key)
                if global_w and global_w["cb"].isChecked():
                    global_ctrl = global_w["control"]
                    gv = global_ctrl.text() if isinstance(global_ctrl, QLineEdit) else global_ctrl.currentText()
                    if gv and gv.strip():
                        nav_val = gv
                        nav_source = "global override"
            if not nav_val or not nav_val.strip():
                nav_val = stock_val
                nav_source = "stock value"
            if nav_val and nav_val.strip():
                btn_link.setEnabled(True)
                btn_link.setToolTip(f"Go to '{nav_val.strip()}' ({nav_source})")
            else:
                btn_link.setEnabled(False)
                btn_link.setToolTip("No sound class assigned")

        # Save updates to DB during edit
        self.save_car_from_ui(car_id, key)

    def save_car_from_ui(self, car_id, key):
        """Saves current UI override value of key for car_id to config dict."""
        if car_id == "[Global Overrides]":
            return

        if car_id not in self.cfg.data["car_overrides"]:
            self.cfg.data["car_overrides"][car_id] = {}

        w_info = self.car_widgets[key]
        if w_info["cb"].isChecked():
            ctrl = w_info["control"]
            val = ctrl.text().replace(",", ".") if isinstance(ctrl, QLineEdit) else ctrl.currentText()
            self.cfg.data["car_overrides"][car_id][key] = val
        else:
            if key in self.cfg.data["car_overrides"][car_id]:
                del self.cfg.data["car_overrides"][car_id][key]

        # Clean empty dicts
        if not self.cfg.data["car_overrides"][car_id]:
            del self.cfg.data["car_overrides"][car_id]

        self.cfg.save_config()

    def action_clear_car_overrides(self):
        current_car = self.list_cars.currentItem()
        if not current_car:
            return
        car_id = current_car.text()

        if car_id == "[Global Overrides]":
            reply = QMessageBox.question(
                self,
                "Clear Global Overrides",
                "Are you sure you want to clear all global overrides?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
            )
            if reply == QMessageBox.StandardButton.No:
                return
            self.cfg.data["global_overrides"] = {}
            self.cfg.save_config()
            # Reset UI checkboxes and inputs
            for key, w_info in self.global_widgets.items():
                w_info["cb"].setChecked(False)
                ctrl = w_info["control"]
                if isinstance(ctrl, QComboBox):
                    ctrl.setCurrentText("")
                elif isinstance(ctrl, QLineEdit):
                    ctrl.setText("")
            self.validate_selected_car_highlights()
            return

        reply = QMessageBox.question(
            self,
            "Clear Overrides",
            f"Are you sure you want to clear all overrides for {car_id.replace('_', ' ')}?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.No:
            return

        if car_id in self.cfg.data["car_overrides"]:
            del self.cfg.data["car_overrides"][car_id]
            self.cfg.save_config()

        # Reload UI for this car
        self.car_selection_changed(current_car)

    # --- Misc Tab Operations ---
    def save_misc_from_ui(self):
        misc_overrides = {}
        if self.cb_misc_blur.isChecked():
            misc_overrides["blur_rim_max_rpm"] = self.edit_misc_blur.text().replace(",", ".")
        if self.cb_misc_bfmin.isChecked():
            misc_overrides["backfire_min_rand_time"] = self.edit_misc_bfmin.text().replace(",", ".")
        if self.cb_misc_bfmax.isChecked():
            misc_overrides["backfire_max_rand_time"] = self.edit_misc_bfmax.text().replace(",", ".")

        self.cfg.data["misc_overrides"] = misc_overrides
        self.cfg.save_config()

    def validate_misc_highlights(self):
        # We check stock misc values and highlight changes
        misc_stock = self.engine.read_stock_misc_values()

        # BlurRim MaxRPM
        is_blur_diff = False
        if self.cb_misc_blur.isChecked():
            curr = self.edit_misc_blur.text()
            stock = misc_stock["blur_rim_max_rpm"]
            try:
                if float(curr) != float(stock):
                    is_blur_diff = True
            except ValueError:
                is_blur_diff = True

        if is_blur_diff:
            self.edit_misc_blur.setStyleSheet("background-color: #3b3b1a; border: 1px solid #d4af37;")
            self.lbl_misc_blur_stock.setStyleSheet("color: #d4af37; font-weight: bold;")
        else:
            self.edit_misc_blur.setStyleSheet("")
            self.lbl_misc_blur_stock.setStyleSheet("color: #a0a0aa; font-weight: normal;")

        # Min delay
        is_min_diff = False
        if self.cb_misc_bfmin.isChecked():
            curr = self.edit_misc_bfmin.text()
            stock = misc_stock["backfire_min_rand_time"]
            try:
                if float(curr) != float(stock):
                    is_min_diff = True
            except ValueError:
                is_min_diff = True

        if is_min_diff:
            self.edit_misc_bfmin.setStyleSheet("background-color: #3b3b1a; border: 1px solid #d4af37;")
            self.lbl_misc_bfmin_stock.setStyleSheet("color: #d4af37; font-weight: bold;")
        else:
            self.edit_misc_bfmin.setStyleSheet("")
            self.lbl_misc_bfmin_stock.setStyleSheet("color: #a0a0aa; font-weight: normal;")

        # Max delay
        is_max_diff = False
        if self.cb_misc_bfmax.isChecked():
            curr = self.edit_misc_bfmax.text()
            stock = misc_stock["backfire_max_rand_time"]
            try:
                if float(curr) != float(stock):
                    is_max_diff = True
            except ValueError:
                is_max_diff = True

        if is_max_diff:
            self.edit_misc_bfmax.setStyleSheet("background-color: #3b3b1a; border: 1px solid #d4af37;")
            self.lbl_misc_bfmax_stock.setStyleSheet("color: #d4af37; font-weight: bold;")
        else:
            self.edit_misc_bfmax.setStyleSheet("")
            self.lbl_misc_bfmax_stock.setStyleSheet("color: #a0a0aa; font-weight: normal;")

        self.save_misc_from_ui()

    # --- Apply Global Profile Action ---
    def action_apply_changes(self):
        has_backup, _ = self.engine.check_backup_status()
        if not has_backup:
            QMessageBox.warning(self, "Warning", "Please create a backup on the Setup tab before applying overrides.")
            return

        # Explicitly save changes from global tab and misc tab to ensure config is up to date
        self.save_global_from_ui()
        self.save_misc_from_ui()

        self.status_bar.showMessage("Applying modifications to active game files...")
        self.progress_bar.setVisible(True)
        self.progress_bar.setValue(0)
        self.setEnabled(False)

        try:
            self.engine.apply_overrides(self.progress_bar.setValue)
            QMessageBox.information(
                self,
                "Success",
                "Overrides successfully applied to game files!\n\nSound overrides have been written to 'media/Audio/ModularCars/' and global attributes to 'media/Cars/_library/GlobalCarAttributes.xml'."
            )
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to apply overrides:\n{e}")
        finally:
            self.setEnabled(True)
            self.progress_bar.setVisible(False)
            self.status_bar.showMessage("Ready")

def main():
    app = QApplication(sys.argv)
    cfg = ConfigManager()
    engine = SoundModifierEngine(cfg)
    window = MainWindow(cfg, engine)
    window.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
