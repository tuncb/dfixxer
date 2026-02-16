unit ComprehensiveTest;

interface

procedure SimpleProc();
function SimpleFunc(): Integer;
procedure ProcWithParams(x: Integer);
function FuncWithParams(x: Integer): String;
procedure ProcWithMultipleParams(x, y: Integer; z: String);

type
  TMyClass = class
    procedure ClassProc();
    function ClassFunc(): Boolean;
    procedure ClassProcWithParams(value: String);
  end;

implementation

procedure SimpleProc();
begin
  // Simple procedure without parameters
end;

function SimpleFunc(): Integer;
begin
  Result := 42;
end;

procedure ProcWithParams(x: Integer);
begin
  // Already has parameters, should not be changed
end;

function FuncWithParams(x: Integer): String;
begin
  Result := 'test';
end;

procedure ProcWithMultipleParams(x, y: Integer; z: String);
begin
  // Already has multiple parameters, should not be changed
end;

procedure TMyClass.ClassProc();
begin
  // Class method without parameters
end;

function TMyClass.ClassFunc(): Boolean;
begin
  Result := True;
end;

procedure TMyClass.ClassProcWithParams(value: String);
begin
  // Already has parameters, should not be changed
end;

end.