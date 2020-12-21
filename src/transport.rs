/// Implement of the data link and physical layer
///
/// - [ ] Ethernet (ISO 8802-3)             Clause 7
/// - [ ] ARCNET (ATA 878.1)                Clause 8
/// - [ ] MS/TP                             Clause 9
/// - [ ] PTP                               Clause 10
/// - [ ] LonTalk (ISO/IEC 14908.1)         Clause 11
/// - [x] BACnet/IP                         Annex J
/// - [ ] BACnet/IPv6                       Annex U
/// - [ ] ZigBee                            Annex O
/// - [ ] BACnet/SC                         Annex YY
///
/// See Figure 4-2. BACnet collapsed architecture.
///
///
pub mod bacnetip;
pub mod bacnetsc;
