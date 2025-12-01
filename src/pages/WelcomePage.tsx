import { useEffect, useState } from "react";
import lightLogo from "./../assets/PixusLogoHD.png";
import darkLogo from "./../assets/PixusLogoHDDarkmode.png";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import { Box, Button,  CircularProgress,  FormControlLabel,  FormGroup,  Paper, Stack, Switch, Table, TableBody, TableCell, TableContainer, TableHead, TablePagination, TableRow, TextField, Typography, useTheme } from "@mui/material";
import Layout from './../Layout';
import { useNavigate } from "react-router-dom";
import SettingsIcon from '@mui/icons-material/Settings';

type Order = {
  order_number: string;
  order_number_full: string;
  part_number: string;
  due_quantity: number;
  assn_number: string;
};

function WelcomePage() {
  const [orderNumber, setOrderNumber] = useState("");

  const orderNumValid = orderNumber.length >= 8;
  const [orders, setOrders] = useState<Order[] | null>(null);
  
  const [ordersFiltered, setOrdersFiltered] = useState<Order[] | null>(null);

  const [page, setPage] = useState(0);
  const rowsPerPage = 5 ;

  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  
  const [check5xx, setCheck5xx] = useState<boolean>(true);
  const [check7xx, setCheck7xx] = useState<boolean>(false);

  const handleCheck5xx = (event: React.ChangeEvent<HTMLInputElement>) => {
    setCheck5xx(event.target.checked);
  };
  const handleCheck7xx = (event: React.ChangeEvent<HTMLInputElement>) => {
    setCheck7xx(event.target.checked);
  };



  const navigate = useNavigate();

  useEffect(() => {
    invoke<Order[]>('get_orders', { })
        .then((data) => {
          setOrders(data);
          console.log(data)
        })
        .catch((error) => {
            console.error("Error fetching orders:", error);
        });
    
    invoke<String>('reset_serial_check', { })
        .then((data) => {
          console.log(data)
        })
        .catch((error) => {
            console.error("Error resetting serial:", error);
        });
  }, [])

  useEffect(() => {
    updateFilteredOrders();
  }, [orders])

  useEffect(() => {
    updateFilteredOrders();
  }, [check5xx, check7xx])

  const updateFilteredOrders = () => {
    if (orders){
      const remainingOrders = orders.filter((order) => (order.order_number.startsWith("5") && check5xx) || (order.order_number.startsWith("7") && check7xx) );
      setOrdersFiltered(remainingOrders);
    }
  }

  const handleChangePage = (_event: unknown, newPage: number) => {
    setPage(newPage);
  };

  const handleTableClick = (orderNum: String) => {
    const welcomeState = { orderNumber: orderNum };
    navigate('/main', {state: welcomeState} );
  }

  const handleClick = () => {
    const welcomeState = { orderNumber: orderNumber };
    navigate('/main', {state: welcomeState} );
  }

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    const welcomeState = { orderNumber: orderNumber };
    if (event.key === "Enter" && orderNumValid) {
      navigate('/main', {state: welcomeState} );
    }
  };

  const handleSettings = () => {
    navigate('/settings');
  }


  return (
    <Layout>
      <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: '1.75em'}}>
        <Box sx={{ position: 'relative', display: 'flex', justifyContent: 'center', alignItems: 'center', flex: 1, width: '100%', gap: '2em' }}>
            <img src={isDarkMode ? darkLogo : lightLogo} alt="Logo" style={{ height: '4em' }} />
          <Box sx={{ display: 'flex', alignItems: 'center', gap: '0.5em', overflowY: 'auto' }}>
            <Typography variant="h4">OrderScanningPal</Typography>
          </Box>
          <SettingsIcon  onClick={handleSettings} sx={{ cursor: 'pointer', fontSize: '2em', marginLeft: '3em' }} />
        </Box>

        { ordersFiltered ? (
          <Box 
            sx={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'flex-start',
              width: '100%' 
            }}
          >
            <Stack direction="column" spacing={2} sx={{ position: 'absolute', left: '10em', mt: '5em'}}>
              <FormGroup>
                <FormControlLabel control={<Switch checked={check5xx} onChange={handleCheck5xx} />} label="5xx's" />
                <FormControlLabel control={<Switch checked={check7xx} onChange={handleCheck7xx} />} label="7xx's" />
              </FormGroup>
            </Stack>
            <Box sx={{minHeight: '25em'}}> 
              <Box sx={{ alignItems: 'center',  width: '30em'}}>
                <TableContainer component={Paper}>
                    <Table>
                        <TableHead>
                        <TableRow>
                            <TableCell>Order Number</TableCell>
                            <TableCell>Part Number</TableCell>
                        </TableRow>
                        </TableHead>
                        <TableBody>
                        {ordersFiltered.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage).map((order, index) => (
                            <TableRow 
                              key={index}
                              hover
                              onClick={(_event) => handleTableClick(order.order_number_full)}
                              role="checkbox"
                              tabIndex={-1}
                              sx={{ cursor: 'pointer' }}
                            >
                            <TableCell>{order.order_number}</TableCell>
                            <TableCell>{order.part_number}</TableCell>
                            </TableRow>
                        ))}
                        </TableBody>
                    </Table>
                </TableContainer>
              </Box>
              <TablePagination
                rowsPerPageOptions={[5]}
                component="div"
                count={ordersFiltered.length}
                rowsPerPage={rowsPerPage}
                page={page}
                onPageChange={handleChangePage}
              />
            </Box>
          </Box>
        
        ) : (
            <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'center', gap: '0.5em', height: '100%', minHeight: '25em'}}>
                <Typography> Loading orders </Typography>
                <CircularProgress />
            </Box>
        )}

        
        <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', height: '6em', gap: '1em'}}> 
          <TextField 
            id="order-number" 
            label="Order Number" 
            variant="outlined" 
            autoComplete="off"
            value={orderNumber}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
              setOrderNumber(event.target.value);
            }}
            onKeyDown={handleKeyDown}
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
