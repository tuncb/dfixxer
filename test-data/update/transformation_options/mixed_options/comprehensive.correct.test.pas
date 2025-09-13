unit TestUnit;

interface

uses
  System.Classes,
  System.SysUtils,
  Vcl.Controls;

procedure TestProc();
function TestFunc(): Integer;

implementation
uses
  System.Math,
  Vcl.Forms;

procedure TestProc();
begin
  // Implementation
end;

function TestFunc(): Integer;
begin
  Result := 42;
end;

initialization
  // Init code

finalization
  // Cleanup code

end.