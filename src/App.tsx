import { CssBaseline, ThemeProvider, createTheme } from '@mui/material';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import WelcomePage from './pages/WelcomePage';
import MainPage from './pages/MainPage';
import SettingsPage from './pages/SettingsPage';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type Settings = {
  font_size: number;
  dark_mode: boolean;
  part_list: string[];
  clr_printer: string;
  bom_path: string;
  snl_path: string;
  config_path: string;
  label_path: string;
  pdf_to_printer_path: string;
  label_printer_125_025: string;
  label_printer_2_025: string;
  label_printer_075_025: string;
  label_printer_2_3: string;
  label_printer_4_6: string;
};

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);

  const loadSettings = () => {
    invoke<Settings>('load_settings')
      .then((data) => setSettings(data))
      .catch((err) => console.error('Failed to load settings:', err));
  };

  useEffect(() => {
    loadSettings();

    const unlisten = listen('settings-updated', () => {
      console.log('Settings update event received');
      loadSettings();
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const theme = createTheme({
    palette: {
      mode: settings?.dark_mode ? 'dark' : 'light',
    },
    typography: {
      fontSize: settings?.font_size ?? 16,
    },
  });
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Routes>
          <Route path="/" element={<WelcomePage />} />
          <Route path="/main" element={<MainPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </Router>
    </ThemeProvider>
  );
}
