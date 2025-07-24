import { useState } from "react";
import pixusLogo from "./../assets/logo.png";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import { Accordion, AppBar, Box, Button, Card, Paper, TextField, Typography } from "@mui/material";
import Diversity1Icon from '@mui/icons-material/Diversity1';
import Layout from './../Layout';
import { useNavigate } from "react-router-dom";

function WelcomePage() {
  const [orderNumber, setOrderNumber] = useState("");

  const orderNumValid = orderNumber.length == 8;

  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }
  const navigate = useNavigate();

  const handleClick = () => {
    // query INFO!
    const welcomeState = { orderNumber: orderNumber };
    // invoke<String>('snu', { serialNumber: "015"})
    //                 .then((data) => {
    //                     // we assume only first bc only 'SHOULD' have 1 order
    //                     console.log("Error fetching orders:", data);
    //                 })
    //                 .catch((error) => {
    //                     console.error("Error fetching orders:", error);
    //                 });
    navigate('/main', {state: welcomeState} );
  }


  return (
    <Layout>
      <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', height: '100%'}}>
        <Box sx={{ display: 'flex', justifyContent: 'space-evenly', gap: '2', alignItems: 'center' }}>
          <img src={pixusLogo} />
         
          <Typography variant="h4"> Welcome to my OrderScanningPal!</Typography>
          {/* <Diversity1Icon></Diversity1Icon> */}

        </Box>

        
        <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', height: '30%'}}> 
          <TextField 
            id="order-number" 
            label="Order Number" 
            variant="outlined" 
            // autoComplete="off"
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

export default WelcomePage;







        {/* <Box
          component="img"
          sx={{
            height: 233,
            width: 350,
            maxHeight: { xs: 233, md: 167 },
            maxWidth: { xs: 350, md: 250 },
          }}
          alt="The house from the offer."
          src="https://images.unsplash.com/photo-1512917774080-9991f1c4c750?auto=format&w=350&dpr=2"
        /> */}
      {/*  
      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p> */}