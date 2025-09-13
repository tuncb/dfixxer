unit TestUnit;

INTERFACE

uses
  System.Classes,
  System.SysUtils,
  Vcl.Controls;

procedure TestProc;
function TestFunc: Integer;

IMPLEMENTATION
uses
  System.Math,
  Vcl.Forms;

procedure TestProc;
begin
  // Implementation
end;

function TestFunc: Integer;
begin
  Result := 42;
end;

INITIALIZATION
  // Init code

FINALIZATION
  // Cleanup code

end.