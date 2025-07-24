import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import pixusLogo from "./../assets/logo.png";
import "./../App.css";
import { Accordion, AppBar, Box, Button, Card, Paper, TextField, Typography, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, } from "@mui/material";
import { DataGrid, GridColDef, GridRowSelectionModel  } from '@mui/x-data-grid';
import Layout from './../Layout';
import { useNavigate, useLocation } from "react-router-dom";
import { ConveyorBelt } from "@mui/icons-material";

type Order = {
  order_number: string;
  part_number: string;
  due_quantity: number;
  assn_number: string;
};

type PrintOrder = {
    order_number: string;
    part_number: string;
    due_quantity: number;
    assn_number: string;
    print_type: string;
    notes: string;
};

type PrintOrderRow = {
    id: number;
    print_type: string;
    notes: string;
};

type PrintStruct = {
    order: Order;
    printOrderRow: PrintOrderRow;
    user: string;
    serialnumber: string;
}

function MainPage() {
  const [order, setOrder] = useState<Order | null>(null);
  const [printOrders, setPrintOrders] = useState<PrintOrder[]>([]);
  const [printOrderRows, setPrintOrderRows] = useState<PrintOrderRow[]>([]);
  const [dueQuantity, setDueQuantity] = useState("");
  const [serialNumber, setSerialNumber] = useState("0"); //pull from document
  const [username, setUsername] = useState("");
  const [rowSelectionModel, setRowSelectionModel] = useState<GridRowSelectionModel>({ type: 'include', ids: new Set() });
  const navigate = useNavigate();
  const location = useLocation();

  

    // const [selectionModel, setSelectionModel] = useState<GridRowSelectionModel>([]);
    
    // console.log('selectionModel:', selectionModel);

    // // Access selected rows by filtering
    const selectedOrders = printOrderRows.filter((row) =>
        (rowSelectionModel).ids.has(row.id)
    );
    

    const usernameFilled = username.length > 1;
    const paginationModel = { page: 0, pageSize: 5 };
    const loaded = printOrderRows.length > 0;

    useEffect(() => {
        invoke<string>('get_serial_number', { } )
            .then((data) => {
                setSerialNumber(data);
            })
            .catch((error) => {
                console.error("Error fetching orders:", error);
            });
    }, [])

    useEffect(() => {
        if(location.state.orderNumber.length == 8) {
            invoke<Order[]>('get_order_number_info', { orderNumber: location.state.orderNumber })
                .then((data) => {
                    // we assume only first bc only 'SHOULD' have 1 order
                    setOrder(trimOrderFields(data[0]));
                    setDueQuantity(data[0].due_quantity.toString());
                })
                .catch((error) => {
                    console.error("Error fetching orders:", error);
                });
            invoke<PrintOrder[]>('get_print_items', { orderNumber: location.state.orderNumber })
                .then((data) => {
                    // we assume only first bc only 'SHOULD' have 1 order
                    const trimmedData = data.map(trimPrintOrderFields)
                    setPrintOrders(trimmedData);
                    let i = 1;
                    setPrintOrderRows(trimmedData.map(order => ({
                        id: i++,
                        print_type: order.print_type,
                        notes: order.notes,
                    })));

                    console.log(trimmedData);
                })
                .catch((error) => {
                    console.error("Error fetching print orders:", error);
                });
            
        }
    }, [location.state.orderNumber])

    const trimOrderFields = (order: Order): Order => ({
        ...order,
        order_number: order.order_number.trim(),
        part_number: order.part_number.trim(),
        assn_number: order.assn_number.trim(),
    });

    const trimPrintOrderFields = (order: PrintOrder): PrintOrder => ({
        ...order,
        order_number: order.order_number.trim(),
        part_number: order.part_number.trim(),
        assn_number: order.assn_number.trim(),
        print_type: order.print_type.trim(),
        notes: order.notes.trim(),
    });
    
    const handleQuantityDown = () => {
        setDueQuantity((parseInt(dueQuantity)-1).toString());
    }

    const handleQuantityUp = () => {
        setDueQuantity((parseInt(dueQuantity)+1).toString());
    }

    const handleSerialNumberDown = () => {
        setSerialNumber('0'+(parseInt(serialNumber)-1).toString());
    }

    const handleSerialNumberUp = () => {
        setSerialNumber('0'+(parseInt(serialNumber)+1).toString());
    }

    const handlePrint = () => {
        for (const rowOrder of selectedOrders) {
            invoke<string>('print', { input: {
                order: {
                    order_number: location.state.orderNumber,
                    part_number: order?.part_number || "",
                    due_quantity: order?.due_quantity || 0,
                    assn_number: order?.assn_number || ""
                },
                printOrderRow: {
                    id: rowOrder?.id || 0,
                    print_type: rowOrder?.print_type || "",
                    notes: rowOrder?.notes || ""
                },
                user: username || "",
                serialnumber: serialNumber

            } })
                .then((data) => {
                    console.log("success for ", rowOrder.print_type)
                })
                .catch((error) => {
                    console.error("Error fetching orders:", error);
                });
        }
        
        console.log(rowSelectionModel, selectedOrders);
        // navigate('/done');
    }

    const handleCancel = () => {
        navigate('/');
    }

    const columns: GridColDef[] = [
        { field: 'print_type', headerName: 'Print Type', width: 170 },
        { field: 'notes', headerName: 'Notes', width: 530, 
            sortable: false,
            description: 'The note section of the BOM', 
        },
        // {
        //     field: 'fullName',
        //     headerName: 'Full name',
        //     description: 'This column has a value getter and is not sortable.',
        //     sortable: false,
        //     width: 160,
        //     valueGetter: (value, row) => `${row.firstName || ''} ${row.lastName || ''}`,
        // },
    ];

    

  return (
      <Layout>
            <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', gap: 2, height: '100%'}}>
                <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center',  width: '100%'}}>
                    <Typography variant="h4">Order Info:</Typography>
                    <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', width: '100%'}}>
                        <Typography>Order Number: {order?.order_number}</Typography>
                        <Typography>Due Quantity: {order?.due_quantity}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', width: '100%'}}>
                        <Typography>Part Number: {order?.part_number}</Typography>
                        <Typography>Assn Number: {order?.assn_number}</Typography>
                    </Box>
                </Box>
                
                <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', gap: 4}}> 
                    <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: 1}}>
                        <Button  //could restrict input to num only
                            id="print-minus-button" 
                            variant="outlined"
                            onClick={handleQuantityDown}
                        >
                        -
                        </Button>
                        <TextField 
                            id="print-amount-textfield" 
                            label="Print Quantity" 
                            variant="outlined" 
                            value={dueQuantity}
                            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                                setDueQuantity(event.target.value);
                            }}
                            sx={{width: 150}}
                            
                        />
                        <Button  //could restrict input to num only
                            id="print-plus-button" 
                            variant="outlined"
                            onClick={handleQuantityUp}
                        >
                        +
                        </Button>
                    </Box>

                    <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: 1}}>
                        <Button  //could restrict input to num only
                            id="serial-number-minus-button" 
                            variant="outlined"
                            onClick={handleSerialNumberDown}
                        >
                        -
                        </Button>
                        <TextField 
                            id="serial-number-textfield" 
                            label="Starting Serial Number" 
                            variant="outlined" 
                            value={serialNumber}
                            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                                setSerialNumber(event.target.value);
                            }}
                            sx={{width: 150}}
                        />
                        <Button  //could restrict input to num only
                            id="serial-number-plus-button" 
                            variant="outlined"
                            onClick={handleSerialNumberUp}
                        >
                        +
                        </Button>
                    </Box>
                </Box>

                { loaded && 
                <Card sx={{ display: 'flex',  alignItems: 'center',  height: 370,}}> 
                    {/* <TableContainer component={Paper}>
                        <Table>
                            <TableHead>
                            <TableRow>
                                <TableCell>Print Type</TableCell>
                                <TableCell>Note</TableCell>
                            </TableRow>
                            </TableHead>
                            <TableBody>
                            {printOrders.map((order, index) => (
                                <TableRow key={index}>
                                <TableCell>{order.print_type}</TableCell>
                                <TableCell>{order.notes}</TableCell>
                                </TableRow>
                            ))}
                            </TableBody>
                        </Table>
                    </TableContainer> */}
                    <DataGrid
                        rows={printOrderRows}
                        columns={columns}
                        initialState={{ pagination: { paginationModel } }}
                        pageSizeOptions={[5]}
                        checkboxSelection
                        onRowSelectionModelChange={(newRowSelectionModel) => {
                            setRowSelectionModel(newRowSelectionModel);
                        }}
                        rowSelectionModel={rowSelectionModel}
                        sx={{ border: 0 }}
                    />
                </Card>
                }
              
                <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', gap: 2}}> 
                    <TextField 
                        id="name-textfield" 
                        label="Username" 
                        // autoComplete="off"
                        variant="outlined" 
                        value={username}
                        onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                            setUsername(event.target.value);
                        }}
                        required
                    />

                    <Button  //could restrict input to num only
                        id="print-button" 
                        variant="outlined"
                        onClick={handlePrint}
                        disabled={!usernameFilled}
                    >
                        Print
                    </Button>

                    <Button  //could restrict input to num only
                        id="cancel-button" 
                        variant="outlined"
                        onClick={handleCancel}
                    >
                        Cancel
                    </Button>
                    
                </Box>
      
            </Box>
        </Layout>
  );
}

export default MainPage;
