import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./../App.css";
import {Box, Button, Card, TextField, Typography, CircularProgress, Checkbox, } from "@mui/material";
import { DataGrid, GridColDef, GridRowSelectionModel  } from '@mui/x-data-grid';
import Layout from './../Layout';
import { useNavigate, useLocation } from "react-router-dom";
import confetti from 'canvas-confetti';
import MyAlert, { SnackbarMessage } from "../components/MyAlert";


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

function MainPage() {
  const [order, setOrder] = useState<Order | null>(null);
  const [printOrderRows, setPrintOrderRows] = useState<PrintOrderRow[]>([]);
  const [dueQuantity, setDueQuantity] = useState("");
  const [serialNumber, setSerialNumber] = useState("0"); //pull from document
  const [username, setUsername] = useState("");
  const [rowSelectionModel, setRowSelectionModel] = useState<GridRowSelectionModel>({ type: 'include', ids: new Set() });
  const [snackPack, setSnackPack] = useState<readonly SnackbarMessage[]>([]);
  const [open, setOpen] = useState(false);
  const [messageInfo, setMessageInfo] = useState<SnackbarMessage | undefined>(
    undefined,
  );
  const [errorPrintAmount, setErrorPrintAmount] = useState("");
  const [errorSerialNumber, setErrorSerialNumber] = useState("");
  const [reprintRun, setReprintRun] = useState(Boolean);
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
                console.error("Error getting serial numbers:", error);
            });
    }, [])

    useEffect(() => {
        if(location.state.orderNumber.length >= 8) {
            invoke<Order[]>('get_order_number_info', { orderNumber: location.state.orderNumber })
                .then((data) => {
                    // we assume only first bc only 'SHOULD' have 1 order
                    setOrder(trimOrderFields(data[0]));
                    setDueQuantity(data[0].due_quantity.toString());
                    console.log(data)
                })
                .catch((error) => {
                    console.error("Error fetching order:", error);
                });
            invoke<PrintOrder[]>('get_print_items', { orderNumber: location.state.orderNumber })
                .then((data) => {
                    const trimmedData = data.map(trimPrintOrderFields)
                    let i = 1;
                    setPrintOrderRows(trimmedData.map(order => ({
                        id: i++,
                        print_type: order.print_type,
                        notes: order.notes,
                    })));

                    console.log(trimmedData);
                })
                .catch((error) => {
                    console.error("Error fetching print order:", error);
                });
            
        }
    }, [location.state.orderNumber])

    

    useEffect(() => {
        if (serialNumber.endsWith("69")) {
            fireConfetti();
        }
    }, [serialNumber])

    const fireConfetti = () => {
        confetti({
            particleCount: 500,
            spread: 150,
            drift: -1,
            ticks: 400,
            origin: { y: 0.2, x: 0.8 },
        });
        // confetti({
        //     particleCount: 500,
        //     angle: -135,
        //     spread: 130,
        //     drift: -1,
        //     ticks: 400,
        //     origin: { y: 0.0, x: 1 },
        // });
        // confetti({
        //     particleCount: 100,
        //     spread: 70,
        //     origin: { y: 0.6 },
        // });
    };

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
        const originalLength = serialNumber.length;
        const number = parseInt(serialNumber, 10) - 1;
        const newSerial = number.toString().padStart(originalLength, '0');
        setSerialNumber(newSerial);
    }

    const handleSerialNumberUp = () => {
        const originalLength = serialNumber.length;
        const number = parseInt(serialNumber, 10) + 1;
        const newSerial = number.toString().padStart(originalLength, '0');
        setSerialNumber(newSerial);
    }

    const handlePrint = () => {
        let printSuccessCount = selectedOrders.length;
        for (const rowOrder of selectedOrders) {
            invoke<string>('print', { 
                order: {
                    order_number: location.state.orderNumber,
                    part_number: order?.part_number || "",
                    due_quantity: parseInt(dueQuantity),
                    assn_number: order?.assn_number || ""
                },
                printOrderRow: {
                    id: rowOrder?.id || 0,
                    print_type: rowOrder?.print_type || "",
                    notes: rowOrder?.notes || ""
                },
                user: username || "",
                serialNumber: serialNumber,
                reprintRun: reprintRun,
            })
                .then((data) => {
                    console.log("rust output", data);
                    const message = "success for " + rowOrder.print_type + " " + rowOrder.notes;
                    console.log(message);
                    printSuccessCount--;
                    if (printSuccessCount == 0) {
                        const message = "Successful print";
                        const type = "success";
                        setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
                        invoke<string>('get_serial_number', { } )
                            .then((data) => {
                                setSerialNumber(data);
                            })
                            .catch((error) => {
                                console.error("Error getting serial numbers:", error);
                            });
                    }

                })
                .catch((error) => {
                    console.error("Error printing:", error);
                    const message = "Error printing: " + rowOrder?.print_type + " Error: " + error;
                    const type = "warning";
                    setSnackPack((prev) => [...prev, { message, type, key: new Date().getTime() }]);
                });
        }
        console.log(rowSelectionModel, selectedOrders);
        // navigate('/done');
    }

    const handleCancel = () => {
        navigate('/');
    }

    const handleStarting = () => {
        const docsString = "initial docs";
        // if one of them aren't selected select all
        if (selectedOrders.filter((order) => order.id <= 3 || order.print_type.toLocaleLowerCase() == docsString ).length 
            < printOrderRows.filter((order) => order.print_type.toLocaleLowerCase() == docsString ).length + 3) {
            for (const order of printOrderRows.filter((order) =>  order.id <= 3 || order.print_type.toLocaleLowerCase() == docsString )) {
                rowSelectionModel.ids.add(order.id);
            }
        } else {
            //unselect all of them
            for (const order of printOrderRows.filter((order) =>  order.id <= 3 || order.print_type.toLocaleLowerCase() == docsString )) {
                rowSelectionModel.ids.delete(order.id);
            }
        }
        setRowSelectionModel({
            type: rowSelectionModel.type,
            ids: rowSelectionModel.ids,
        });
    }

    const handleLabels = () => {
        const labelsString = "94a";
        const labelsString2 = "k94a";
        // if one of them aren't selected select all
        if (selectedOrders.filter((order) => order.print_type.toLocaleLowerCase().substring(0, 3) == labelsString || order.print_type.toLocaleLowerCase().substring(0, 4) == labelsString2 ).length 
            < printOrderRows.filter((order) => order.print_type.toLocaleLowerCase().substring(0, 3) == labelsString || order.print_type.toLocaleLowerCase().substring(0, 4) == labelsString2 ).length) {
            for (const order of printOrderRows.filter((order) => order.print_type.toLocaleLowerCase().substring(0, 3) == labelsString || order.print_type.toLocaleLowerCase().substring(0, 4) == labelsString2 )) {
                rowSelectionModel.ids.add(order.id);
            }
        } else {
            //unselect all of them
            for (const order of printOrderRows.filter((order) => order.print_type.toLocaleLowerCase().substring(0, 3) == labelsString || order.print_type.toLocaleLowerCase().substring(0, 4) == labelsString2 )) {
                rowSelectionModel.ids.delete(order.id);
            }
        }
        setRowSelectionModel({
            type: rowSelectionModel.type,
            ids: rowSelectionModel.ids,
        });
        
    }

    const handleFinalDocs = () => {
        const docsString = "final docs";
        // if one of them aren't selected select all
        if (selectedOrders.filter((order) => order.print_type.toLocaleLowerCase() == docsString ).length < printOrderRows.filter((order) => order.print_type.toLocaleLowerCase() == docsString ).length) {
            for (const order of printOrderRows.filter((order) => order.print_type.toLocaleLowerCase() == docsString )) {
                rowSelectionModel.ids.add(order.id);
            }
        } else {
            //unselect all of them
            for (const order of printOrderRows.filter((order) => order.print_type.toLocaleLowerCase() == docsString )) {
                rowSelectionModel.ids.delete(order.id);
            }
        }
        setRowSelectionModel({
            type: rowSelectionModel.type,
            ids: rowSelectionModel.ids,
        });
    }

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

    const onChangePrintAmount = (event: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = event.target.value;

        if (newValue.match(/^\d+$/)) {
            setErrorPrintAmount("");
            console.log("no error")
        } else {
            setErrorPrintAmount("Requires a number");
        }
        setDueQuantity(newValue);
    };
    
    const onChangeSerialNumber = (event: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = event.target.value;

        if (newValue.match(/^\d+$/)) {
            setErrorSerialNumber("");
            console.log("no error")
        } else {
            setErrorSerialNumber("Requires a number");
        }
        setSerialNumber(newValue);
    };

    

  return (
                
    <Layout>
        { loaded ? (
        <>
            <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-between', gap: '0.5em', height: '100%'}}>
                <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center',  width: '100%'}}>
                    <Typography variant="h4">Print Selection:</Typography>
                    <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center',  justifyContent: 'space-around', width: '100%', flexWrap: 'wrap'}}>
                        <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', height: '100%', minWidth: 300}}>
                            <Typography>Order Number: {order?.order_number}</Typography>
                            <Typography>Due Quantity: {order?.due_quantity}</Typography>
                            <Typography>Part Number: {order?.part_number}</Typography>
                            <Typography>Assn Number: {order?.assn_number}</Typography>
                        </Box>
                            <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: '0.25em', p: '0.25em'}}>
                                <Button  
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
                                    onChange={onChangePrintAmount}
                                    sx={{width: 150}}
                                    helperText={errorPrintAmount}
                                    error={!!errorPrintAmount}
                                    
                                />
                                <Button  
                                    id="print-plus-button" 
                                    variant="outlined"
                                    onClick={handleQuantityUp}
                                >
                                +
                                </Button>
                            </Box>

                            <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-between', height: '100%', gap: '0.25em', p: '0.25em'}}>
                                <Button
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
                                    onChange={onChangeSerialNumber}
                                    sx={{width: 150}}
                                    helperText={errorSerialNumber}
                                    error={!!errorSerialNumber}
                                />
                                <Button 
                                    id="serial-number-plus-button" 
                                    variant="outlined"
                                    onClick={handleSerialNumberUp}
                                >
                                +
                                </Button>
                            </Box>
                    </Box>
                </Box>

                <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', gap: '2em'}}> 
                    <Box sx={{ display: 'flex', flexDirection: 'column',  alignItems: 'center', justifyContent: 'space-around', gap: '2em'}}> 
                        <Button  
                            id="starting-button" 
                            variant="outlined"
                            onClick={handleStarting}
                        >
                            Starting
                        </Button>

                        <Button 
                            id="labels-button" 
                            variant="outlined"
                            onClick={handleLabels}
                        >
                            Labels
                        </Button>

                        <Button 
                            id="final-docs-button" 
                            variant="outlined"
                            onClick={handleFinalDocs}
                        >
                            Final Docs
                        </Button>
                    </Box>  
                    <Card sx={{ display: 'flex',  alignItems: 'center',  height: '23.25em',}}> 
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
                            // getRowHeight={() => 'auto'}
                            getRowClassName={(params) => {
                                if (params.row.print_type == "BOM" || params.row.print_type == "Config"  || params.row.print_type == "SNL" ) {
                                    return '';
                                } else if (params.row.print_type?.startsWith("94A") || params.row.print_type?.startsWith("K94A")) {
                                    const valueToValidate = params.row.notes; 
                                    const isValid = /^01A[0-9]{6}-[A-Z][0-9]{2}(\?.+){0,4}$/.test(valueToValidate); //checks for 0 or 4 params
                                    return isValid ? '' : 'invalid-row';
                                } else if (params.row.print_type == "Final DOCS" || params.row.print_type == "INITIAL DOCS") {
                                    const valueToValidate = params.row.notes;
                                    const isValid = /^[A-Z]:\\[^?]+(\?.+){1,2}$/.test(valueToValidate); // checks for 1 or 2 params
                                    return isValid ? '' : 'invalid-row';
                                }
                                return 'invalid-row';
                            }}
                        />
                    </Card>
                </Box>
                
              
                <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'space-around', gap: '1em', p: '0.5em' }}> 
                    <Box >
                        <Checkbox sx={{ 'aria-label': 'Checkbox demo' }} 
                            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                                setReprintRun(event.target.checked);
                            }}
                            value={reprintRun}
                            disabled={selectedOrders.filter((order) => order.id > 3 && order.print_type != "INITIAL DOCS").length == 0} // if no labels/final docs selected disable
                        />
                        Reprint Run
                    </Box>
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

                    <Button 
                        id="print-button" 
                        variant="outlined"
                        onClick={handlePrint}
                        disabled={!usernameFilled}
                    >
                        Print
                    </Button>

                    <Button 
                        id="cancel-button" 
                        variant="outlined"
                        onClick={handleCancel}
                    >
                        Cancel
                    </Button>
                    
                </Box>
      
            </Box>
            <MyAlert
                open={open}
                setOpen={setOpen}
                messageInfo={messageInfo}
                setMessageInfo={setMessageInfo}
            />
        </>
        ) : (
            <Box sx={{ display: 'flex', flexDirection: 'row',  alignItems: 'center', justifyContent: 'center', gap: '1em', height: '100%', minHeight: '38em'}}>
                <Typography> Loading your order </Typography>
                <CircularProgress />
                
            </Box>
        )}
    </Layout>
                    
  );
}

export default MainPage;
