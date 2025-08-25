# OrderScanningPal

This guide provides instructions for setting up and installing the OrderScanningPal application, including configuring the `.env` files and performing the necessary installation steps.

## Environment File Setup

You need to configure the following `.env` files for your development and production environments:

### `.env.dev` (Development)
Create a `.env.dev` file in the project root with the following database configuration:
```
DB_HOST=your_database_host
DB_PORT=your_database_port
DB_USER=your_database_user
DB_PASSWORD=your_database_password
DB_NAME=your_database_name
```
### `.env.prod` (Production)
Create a `.env.prod` file in the project root with the following database configuration:
```
DB_HOST=your_database_host
DB_PORT=your_database_port
DB_USER=your_database_user
DB_PASSWORD=your_database_password
DB_NAME=your_database_name
```

Replace `your_database_host`, `your_database_port`, `your_database_user`, `your_database_password`, and `your_database_name` with the appropriate values for your database.

### DOC_PATH 
Finally, add a DOC_PATH variable, 
if you are in .prod, set to build
otherwise, set to a folder location where you want the files to be, such as the Serial Number List, and the app settings.

## Installation Steps

1. **Build the Application**
   - Run the following command to build the application:
     ```bash
     npm run tauri-build
     ```
   - Once the build is complete, navigate to the `src-tauri/target/release/bundle` directory and choose your preferred installation method 

2. **Install Barcode Software**
   - Install the required barcode software for Crystal Reports. The installer can be found in the `/installs` directory.
   - Follow the software’s installation instructions to set it up correctly.

3. **Install PDFtoPrinter**
   - Download and install the `PDFtoPrinter` software, which is required for printing functionality.
   - Ensure the software is properly configured before proceeding.

4. **Configure VC11 and Database**
   - Open the VC11 application and sign into the database using the credentials specified in your `.env` file.
   - **Note**: Printing will not work unless you are signed into the database.

5. **Set Up Default Printer**
   - On Windows, ensure the default printer is configured to the desired printer for your output.
   - Verify the printer settings in the Windows Control Panel under "Devices and Printers."

## Key Notes
- Ensure all dependencies (barcode software, `PDFtoPrinter`, and VC11) are installed and configured correctly before running the application.
- Double-check the database credentials in the `.env.dev` or `.env.prod` files to avoid connection issues.
- If printing issues occur, verify that you are signed into the database via VC11 and that the default printer is correctly set.