import React from 'react';
import Snackbar, { SnackbarCloseReason } from '@mui/material/Snackbar';
import Alert, { AlertColor } from '@mui/material/Alert';


export type SnackbarMessage = {
  message: string;
  type: AlertColor;
  key: number;
};

interface MyAlertProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  messageInfo: SnackbarMessage | undefined;
  setMessageInfo: (info: SnackbarMessage | undefined) => void;
  autoHideDuration?: number;
}

const MyAlert: React.FC<MyAlertProps> = ({
  open,
  setOpen,
  messageInfo,
  setMessageInfo,
  autoHideDuration = 5000,
}) => {

  const handleClose = (
      _event: React.SyntheticEvent | Event,
      reason?: SnackbarCloseReason,
  ) => {
      if (reason === 'clickaway') {
      return;
      }
      setOpen(false);
  };
      
  const handleExited = () => {
      setMessageInfo(undefined);
  };
  return (
    <Snackbar
      key={messageInfo ? messageInfo.key : undefined}
      open={open}
      autoHideDuration={autoHideDuration}
      onClose={handleClose}
      slotProps={{ transition: { onExited: handleExited } }}
    >
      <Alert
        onClose={handleClose}
        severity={messageInfo ? messageInfo.type : "info"}
        variant="filled"
        sx={{ width: '100%' }}
      >
        {messageInfo ? messageInfo.message : undefined}
      </Alert>
    </Snackbar>
  );
};

export default MyAlert;
