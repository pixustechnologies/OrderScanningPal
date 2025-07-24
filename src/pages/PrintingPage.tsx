import { useState } from "react";
import pixusLogo from "./../assets/logo.png";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import { Accordion, AppBar, Box, Button, Card, Paper, TextField, Typography } from "@mui/material";
import Layout from './../Layout';
import { useNavigate } from "react-router-dom";

function PrintingPage() {
  const [orderNumber, setOrderNumber] = useState("");

  const orderNumValid = orderNumber.length == 8;

  const navigate = useNavigate();

  const handleClick = () => {
    navigate('/');
  }


  return (
    <Layout>
      <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', height: '100%'}}>
        <Box sx={{ display: 'flex', justifyContent: 'space-evenly', gap: '2', alignItems: 'center' }}>
          <img src={pixusLogo} />
         
          <Typography variant="h4">Printing!</Typography>

        </Box>

        
        <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', height: '30%'}}> 
          <TextField 
            id="order-number" 
            label="Order Number" 
            variant="outlined" 
            value={orderNumber}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
              setOrderNumber(event.target.value);
            }}
            required
          />
          
          <Button  //could restrict input to num only
            id="continue-button" 
            variant="outlined"
            disabled={!orderNumValid}
            onClick={handleClick}
          >
            Continue
          </Button>
        </Box>
        



      </Box>
    </Layout>
  );
}

export default PrintingPage;
