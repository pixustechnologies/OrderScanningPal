import { useEffect, useState } from "react";
import pixusLogo from "./../assets/logo.png";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import { Box, Button,  CircularProgress,  Paper, Table, TableBody, TableCell, TableContainer, TableHead, TablePagination, TableRow, TextField, Typography } from "@mui/material";
import Layout from './../Layout';
import { useNavigate } from "react-router-dom";

type Order = {
  order_number: string;
  part_number: string;
  due_quantity: number;
  assn_number: string;
};

function WelcomePage() {
  const [orderNumber, setOrderNumber] = useState("");

  const orderNumValid = orderNumber.length == 8;
  const [orders, setOrders] = useState<Order[] | null>(null);

  const [page, setPage] = useState(0);
  const rowsPerPage = 5 ;

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
            
  }, [])

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


  return (
    <Layout>
      <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', height: '100%'}}>
        <Box sx={{ display: 'flex', justifyContent: 'space-evenly', gap: 4, alignItems: 'center' }}>
          <img src={pixusLogo} />
         
          <Typography variant="h4"> Welcome to my OrderScanningPal!</Typography>
          {/* <Diversity1Icon></Diversity1Icon> */}

        </Box>

        { orders ? (
        <Box>
        <Box sx={{ alignItems: 'center',  width: 480}}>
          <TableContainer component={Paper}>
              <Table>
                  <TableHead>
                  <TableRow>
                      <TableCell>Order Number</TableCell>
                      <TableCell>Part Number</TableCell>
                  </TableRow>
                  </TableHead>
                  <TableBody>
                  {orders.slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage).map((order, index) => (
                      <TableRow 
                        key={index}
                        hover
                        onClick={(_event) => handleTableClick(order.order_number)}
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
            count={orders.length}
            rowsPerPage={rowsPerPage}
            page={page}
            onPageChange={handleChangePage}
          />
        
        </Box>
        
        ) : (
            <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'center', gap: 2, height: '100%'}}>
                <Typography> Loading orders </Typography>
                <CircularProgress />
            </Box>
        )}

        
        <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', height: '20%'}}> 
          <TextField 
            id="order-number" 
            label="Order Number" 
            variant="outlined" 
            // autoComplete="off"
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