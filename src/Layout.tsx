// Layout.tsx
import React, { useEffect, useState } from 'react';
import { Box, Paper } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

type Settings = {
  font_size: number;
  dark_mode: boolean;
  part_list: String[];
};

export default function Layout({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<Settings>({font_size: 16, dark_mode: false, part_list: []});


    useEffect(() => {
      invoke<Settings>('load_settings', { })
          .then((data) => {
            setSettings(data);
            console.log(data)
          })
          .catch((error) => {
              console.error("Error fetching settings:", error);
          });
              
    }, [])
    
  return (
    <Box
      sx={{
        backgroundColor: '#1971A8', // or use theme.palette.grey[100]
        minWidth: '100%',
        minHeight: '100vh',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        padding: 2,
        fontSize: `${settings.font_size}px`,
      }}
    >
      <Paper
        elevation={3}
        sx={{
          width: '100%',
        //   maxWidth: 'auto',
          height: '100%',
          padding: 4,
          boxSizing: 'border-box',
        }}
      >
        {children}
      </Paper>
    </Box>
  );
}
