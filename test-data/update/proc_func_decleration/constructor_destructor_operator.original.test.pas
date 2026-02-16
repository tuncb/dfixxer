unit TestCtorDtorOperator;

interface

type
  TMyRecord = record
    class operator Negative: TMyRecord;
  end;

  TBaseClass = class
  public
    procedure Reset; virtual;
  end;

  TMyClass = class(TBaseClass)
  public
    constructor Create;
    destructor Destroy; override;
    procedure Reset; override;
  end;

implementation

procedure TBaseClass.Reset;
begin
end;

constructor TMyClass.Create;
begin
end;

destructor TMyClass.Destroy;
begin
  inherited;
end;

procedure TMyClass.Reset;
begin
end;

class operator TMyRecord.Negative: TMyRecord;
begin
end;

end.