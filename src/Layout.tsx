// Layout.tsx
import React from 'react';
import { Box, Paper } from '@mui/material';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <Box
      sx={{
        backgroundColor: '#1971A8', // or use theme.palette.grey[100]
        width: '100vw',
        height: '100vh',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        padding: 2,
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
