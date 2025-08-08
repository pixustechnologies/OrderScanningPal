import { useEffect, useState } from "react";
import lightLogo from "./../assets/PixusLogoHD.png";
import darkLogo from "./../assets/PixusLogoHDDarkmode.png";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import { Box, Button,  CircularProgress,  Paper, TextField, Typography, List, ListItem, ListItemText, Switch, IconButton, ListItemSecondaryAction, Divider, useTheme } from "@mui/material";
import Layout from './../Layout';
import { useNavigate } from "react-router-dom";
import KeyboardBackspaceIcon from '@mui/icons-material/KeyboardBackspace';
import DeleteIcon from '@mui/icons-material/Delete';
import AddIcon from '@mui/icons-material/Add';
import MyAlert, { SnackbarMessage } from "../components/MyAlert";


type Settings = {
  font_size: number;
  dark_mode: boolean;
  part_list: String[];
};

// font size, avoid list, dark mode
function SettingsPage() {
  const [settings, setSettings] = useState<Settings| null>(null);
  const [darkMode, setDarkMode] = useState<boolean>(false);
  const [fontSize, setFontSize] = useState<number>(16);
  const [partList, setPartList] = useState<String[]>([]);

  const [inputValue, setInputValue] = useState('');
  const [snackPack, setSnackPack] = useState<readonly SnackbarMessage[]>([]);
  const [open, setOpen] = useState(false);
  const [messageInfo, setMessageInfo] = useState<SnackbarMessage | undefined>(
    undefined,
  );
  
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  const handleAdd = () => {
    const trimmed = inputValue.trim();
    if (trimmed) {
      setPartList((prev) => [...prev, trimmed]);
      setInputValue('');
    }
  };

  const handleRemove = (index: number) => {
    setPartList((prev) => prev.filter((_, i) => i !== index));
  };

  const navigate = useNavigate();

  useEffect(() => {
    invoke<Settings>('load_settings', { })
        .then((data) => {
          setSettings(data);
          console.log(data)
        })
        .catch((error) => {
          console.error("Error fetching settings:", error); // throw out 
          const message = "Error collecting settings: " + error;
          const type = "warning";
          setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
        });
            
  }, [])

  useEffect(() => {
    if (settings){
      setDarkMode(settings.dark_mode);
      setFontSize(settings.font_size);
      setPartList(settings.part_list);
    }
  }, [settings]);

  useEffect(() => {
      if (snackPack.length && !messageInfo) {
          // Set a new snack when we don't have an active one
          setMessageInfo({ ...snackPack[0] });
          setSnackPack((prev) => prev.slice(1));
          setOpen(true);
      } else if (snackPack.length && messageInfo && open) {
          // Close an active snack when a new one is added
          setOpen(false);
      }
  }, [snackPack, messageInfo, open]);

  const handleSave = () => {
    const newSettings = {
      dark_mode: darkMode,
      font_size: fontSize,
      part_list: partList,
    }
    invoke<Settings>('save_settings', { settings: newSettings })
        .then((data) => {
          console.log(data)
          const message = "Success saving settings";
          const type = "success";
          setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
        })
        .catch((error) => {
            console.error("Error saving settings:", error);
            const message = "Error saving settings: " + error;
            const type = "warning";
            setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
        });
  }
  
  const handleUndo = () => {
    invoke<Settings>('load_settings', { })
        .then((data) => {
          setSettings(data);
          console.log(data)
        })
        .catch((error) => {
            console.error("Error fetching settings:", error);
            const message = "Error undoing settings: " + error;
            const type = "warning";
            setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
        });
  }
  
  const handleBack = () => {
    navigate('/');
  }

  const handleDarkMode = () => {
    setDarkMode(!darkMode);
  }

  

  return (
    <Layout>
      <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: '2em'}}>
        <Box sx={{ position: 'relative', display: 'flex', justifyContent: 'center', alignItems: 'center', height: '4em', width: '100%' }}>
          {/* Settings icon aligned to the left */}
          <Box sx={{ position: 'absolute', left: 16 }}>
            <KeyboardBackspaceIcon onClick={handleBack} sx={{ cursor: 'pointer', fontSize: '2em' }} ></KeyboardBackspaceIcon>
          </Box>
          {/* Centered content */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: '1em' }}>
            <img src={isDarkMode ? darkLogo : lightLogo} alt="Logo" style={{ height: '4em' }} />
            <Typography variant="h4">Settings</Typography>
          </Box>
        </Box>

        { settings ? (
        <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: '1em'}}>
          <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', gap: '1em'}}>
            <Box>
              Dark Mode
              <Switch id="dark-mode-switch" checked={darkMode} onChange={handleDarkMode} /> 
            </Box>
            <Box sx={{p: '1em'}}>
              <TextField 
                id="font-size-textfield" 
                label="Font Size" 
                variant="outlined" 
                autoComplete="off"
                value={fontSize}
                onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                  setFontSize(parseInt(event.target.value));
                }}
              />
            </Box>
          </Box>
            <Paper elevation={1} sx={{ p: '1.5em', pb: '0.5em', maxWidth: '25em',  mx: 'auto', minHeight: '28em' }}>
                Labels to Omit
              {/* Add New Item */}
              <Box sx={{ display: 'flex', gap: '1em', mb: '1em' }}>
                <TextField
                  fullWidth
                  label="New label"
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAdd()}
                />
                <IconButton
                  color="primary"
                  onClick={handleAdd}
                  aria-label="add"
                >
                  <AddIcon />
                </IconButton>
              </Box>

              <Divider />

              {/* List Display */}
              <Box sx={{maxHeight: '18em', overflowY: 'auto'}}>
              <List>
                {partList.map((item, index) => (
                  <ListItem key={index} >
                    <ListItemText primary={item} />
                    <ListItemSecondaryAction>
                      <IconButton
                        edge="end"
                        aria-label="delete"
                        onClick={() => handleRemove(index)}
                      >
                        <DeleteIcon />
                      </IconButton>
                    </ListItemSecondaryAction>
                  </ListItem>
                ))}
                {partList.length === 0 && (
                  <Typography variant="body2" sx={{ mt: 2, color: 'text.secondary' }}>
                    No labels avoided.
                  </Typography>
                )}
              </List>
              </Box>
            </Paper>
        </Box>
        ) : (
          <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'center', gap: '1em', height: '100%', minHeight: '26em'}}>
              <Typography> Loading settings </Typography>
              <CircularProgress />
          </Box>
        )}

        
        <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', gap: '1em', height: '20%'}}> 
          <Button 
            id="save-button" 
            variant="outlined"
            onClick={handleSave}
          >
            Save Changes
          </Button>
          
          <Button  
            id="undo-button" 
            variant="outlined"
            onClick={handleUndo}
          >
            Undo Changes
          </Button>
        </Box>

      </Box>
      <MyAlert
          open={open}
          setOpen={setOpen}
          messageInfo={messageInfo}
          setMessageInfo={setMessageInfo}
      />
    </Layout>
  );
}

export default SettingsPage;
