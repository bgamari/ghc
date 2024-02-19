{-# LANGUAGE Safe #-}

-- |
--
-- Module      :  Foreign.C.Error
-- Copyright   :  (c) The FFI task force 2001
-- License     :  BSD-style (see the file libraries/base/LICENSE)
--
-- Maintainer  :  ffi@haskell.org
-- Stability   :  provisional
-- Portability :  portable
--
-- C-specific Marshalling support: Handling of C \"errno\" error codes.
--

module Foreign.C.Error
    (-- *  Haskell representations of @errno@ values
     Errno(..),
     -- **  Common @errno@ symbols
     -- | Different operating systems and\/or C libraries often support
     -- different values of @errno@. This module defines the common values,
     -- but due to the open definition of 'Errno' users may add definitions
     -- which are not predefined.
     eOK,
     e2BIG,
     eACCES,
     eADDRINUSE,
     eADDRNOTAVAIL,
     eADV,
     eAFNOSUPPORT,
     eAGAIN,
     eALREADY,
     eBADF,
     eBADMSG,
     eBADRPC,
     eBUSY,
     eCHILD,
     eCOMM,
     eCONNABORTED,
     eCONNREFUSED,
     eCONNRESET,
     eDEADLK,
     eDESTADDRREQ,
     eDIRTY,
     eDOM,
     eDQUOT,
     eEXIST,
     eFAULT,
     eFBIG,
     eFTYPE,
     eHOSTDOWN,
     eHOSTUNREACH,
     eIDRM,
     eILSEQ,
     eINPROGRESS,
     eINTR,
     eINVAL,
     eIO,
     eISCONN,
     eISDIR,
     eLOOP,
     eMFILE,
     eMLINK,
     eMSGSIZE,
     eMULTIHOP,
     eNAMETOOLONG,
     eNETDOWN,
     eNETRESET,
     eNETUNREACH,
     eNFILE,
     eNOBUFS,
     eNODATA,
     eNODEV,
     eNOENT,
     eNOEXEC,
     eNOLCK,
     eNOLINK,
     eNOMEM,
     eNOMSG,
     eNONET,
     eNOPROTOOPT,
     eNOSPC,
     eNOSR,
     eNOSTR,
     eNOSYS,
     eNOTBLK,
     eNOTCONN,
     eNOTDIR,
     eNOTEMPTY,
     eNOTSOCK,
     eNOTSUP,
     eNOTTY,
     eNXIO,
     eOPNOTSUPP,
     ePERM,
     ePFNOSUPPORT,
     ePIPE,
     ePROCLIM,
     ePROCUNAVAIL,
     ePROGMISMATCH,
     ePROGUNAVAIL,
     ePROTO,
     ePROTONOSUPPORT,
     ePROTOTYPE,
     eRANGE,
     eREMCHG,
     eREMOTE,
     eROFS,
     eRPCMISMATCH,
     eRREMOTE,
     eSHUTDOWN,
     eSOCKTNOSUPPORT,
     eSPIPE,
     eSRCH,
     eSRMNT,
     eSTALE,
     eTIME,
     eTIMEDOUT,
     eTOOMANYREFS,
     eTXTBSY,
     eUSERS,
     eWOULDBLOCK,
     eXDEV,
     -- **  'Errno' functions
     isValidErrno,
     getErrno,
     resetErrno,
     errnoToIOError,
     throwErrno,
     -- **  Guards for IO operations that may fail
     throwErrnoIf,
     throwErrnoIf_,
     throwErrnoIfRetry,
     throwErrnoIfRetry_,
     throwErrnoIfMinus1,
     throwErrnoIfMinus1_,
     throwErrnoIfMinus1Retry,
     throwErrnoIfMinus1Retry_,
     throwErrnoIfNull,
     throwErrnoIfNullRetry,
     throwErrnoIfRetryMayBlock,
     throwErrnoIfRetryMayBlock_,
     throwErrnoIfMinus1RetryMayBlock,
     throwErrnoIfMinus1RetryMayBlock_,
     throwErrnoIfNullRetryMayBlock,
     throwErrnoPath,
     throwErrnoPathIf,
     throwErrnoPathIf_,
     throwErrnoPathIfNull,
     throwErrnoPathIfMinus1,
     throwErrnoPathIfMinus1_
     ) where

import GHC.Internal.Foreign.C.Error
